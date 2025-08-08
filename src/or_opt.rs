use crate::shared;
use crate::math;

use std::simd::{Simd};

type SimdF32 = Simd<f32, 8>;

#[inline(always)]
fn calculate_removal_cost(tour: &[shared::Point], start: usize, length: usize) -> f32 {
    let n = tour.len();
    let prev_idx = if start == 0 { n - 1 } else { start - 1 };
    let next_idx = (start + length) % n;
    
    // Cost saved by removing the sequence and connecting prev directly to next
    let old_cost = math::calc_dist(tour[prev_idx], tour[start]) + 
                   math::calc_dist(tour[(start + length - 1) % n], tour[next_idx]);
    let new_cost = math::calc_dist(tour[prev_idx], tour[next_idx]);
    
    old_cost - new_cost
}

fn calculate_insertion_costs_simd(
    tour: &[shared::Point],
    sequence: &[shared::Point],
    excluded_start: usize,
    excluded_length: usize,
) -> Vec<f32> {
    let n = tour.len();
    let seq_len = sequence.len();
    let mut costs = vec![f32::INFINITY; n];
    
    if seq_len == 0 {
        return costs;
    }
    
    let excluded_end = excluded_start + excluded_length;
    
    // Process insertion positions in chunks of 8
    for chunk_start in (0..n).step_by(8) {
        let chunk_end = (chunk_start + 8).min(n);
        let actual_chunk_size = chunk_end - chunk_start;
        
        let mut prev_x = [0.0f32; 8];
        let mut prev_y = [0.0f32; 8];
        let mut next_x = [0.0f32; 8];
        let mut next_y = [0.0f32; 8];
        let mut valid_mask = [false; 8];
        
        for i in 0..actual_chunk_size {
            let pos = chunk_start + i;
            
            // Skip positions within the excluded range
            if pos >= excluded_start && pos < excluded_end {
                continue;
            }
            
            let prev_idx = if pos == 0 { n - 1 } else { pos - 1 };
            let next_idx = pos;
            
            // Skip if prev or next is in excluded range
            if (prev_idx >= excluded_start && prev_idx < excluded_end) ||
               (next_idx >= excluded_start && next_idx < excluded_end) {
                continue;
            }
            
            prev_x[i] = tour[prev_idx].x;
            prev_y[i] = tour[prev_idx].y;
            next_x[i] = tour[next_idx].x;
            next_y[i] = tour[next_idx].y;
            valid_mask[i] = true;
        }
        
        // Load into SIMD
        let prev_x_simd = SimdF32::from_array(prev_x);
        let prev_y_simd = SimdF32::from_array(prev_y);
        let next_x_simd = SimdF32::from_array(next_x);
        let next_y_simd = SimdF32::from_array(next_y);
        
        // Calculate old connection costs (prev -> next)
        let old_costs = math::calc_dist_simd(prev_x_simd, prev_y_simd, next_x_simd, next_y_simd);
        
        // Calculate new connection costs (prev -> seq_start + seq_end -> next)
        let seq_start_x = SimdF32::splat(sequence[0].x);
        let seq_start_y = SimdF32::splat(sequence[0].y);
        let seq_end_x = SimdF32::splat(sequence[seq_len - 1].x);
        let seq_end_y = SimdF32::splat(sequence[seq_len - 1].y);
        
        let new_cost1 = math::calc_dist_simd(prev_x_simd, prev_y_simd, seq_start_x, seq_start_y);
        let new_cost2 = math::calc_dist_simd(seq_end_x, seq_end_y, next_x_simd, next_y_simd);
        let new_costs = new_cost1 + new_cost2;
        
        let cost_diff = new_costs - old_costs;
        
        // Store results for valid positions
        for i in 0..actual_chunk_size {
            if valid_mask[i] {
                costs[chunk_start + i] = cost_diff[i];
            }
        }
    }
    
    costs
}

fn perform_or_opt_move(
    tour: &mut Vec<shared::Point>,
    from_start: usize,
    length: usize,
    to_pos: usize,
) {
    // Extract the sequence to move
    let sequence: Vec<shared::Point> = tour.drain(from_start..from_start + length).collect();
    
    // Adjust insertion position if it's after the removal point
    let adjusted_to_pos = if to_pos > from_start {
        to_pos - length
    } else {
        to_pos
    };
    
    // Insert the sequence at the new position
    for (i, point) in sequence.into_iter().enumerate() {
        tour.insert(adjusted_to_pos + i, point);
    }
}

pub fn or_opt_optimization(hull: &mut Vec<shared::Point>, sequence_length: usize) -> bool {
    let n = hull.len();
    
    if n < sequence_length + 2 {
        return false;
    }
    
    let mut improved = false;
    let max_iterations = 2;
    
    for _iteration in 0..max_iterations {
        let mut found_improvement = false;
        
        // Try each possible sequence position
        for start in 0..(n - sequence_length + 1) {
            // Calculate cost of removing this sequence
            let removal_savings = calculate_removal_cost(hull, start, sequence_length);
            
            // Extract sequence for evaluation
            let sequence: Vec<shared::Point> = hull[start..start + sequence_length].to_vec();
            
            // Calculate insertion costs for all valid positions
            let insertion_costs = calculate_insertion_costs_simd(hull, &sequence, start, sequence_length);
            
            // Find best insertion position
            let mut best_pos = None;
            let mut best_improvement = 0.0;
            
            for (pos, &insertion_cost) in insertion_costs.iter().enumerate() {
                if insertion_cost == f32::INFINITY {
                    continue;
                }
                
                let total_improvement = removal_savings - insertion_cost;
                if total_improvement > best_improvement + 1e-6 {
                    best_improvement = total_improvement;
                    best_pos = Some(pos);
                }
            }
            
            // Perform the best move if it's beneficial
            if let Some(pos) = best_pos {
                perform_or_opt_move(hull, start, sequence_length, pos);
                found_improvement = true;
                improved = true;
                break; // Early termination after first improvement
            }
        }
        
        if !found_improvement {
            break;
        }
    }
    
    return improved
}

// Try multiple sequence lengths
pub fn multi_or_opt_optimization(hull: &mut Vec<shared::Point>) -> bool {
    let mut any_improvement = false;
    
    // Try different sequence lengths, starting with smaller ones
    for seq_len in 1..50 {
        if hull.len() >= seq_len + 2 {
            if or_opt_optimization(hull, seq_len) {
                any_improvement = true;
            }
        }
    }
    
    return any_improvement
}