use std::simd::{Simd};
use std::simd::cmp::SimdPartialOrd;

use crate::math;
use crate::shared;
type SimdBool = std::simd::Mask<i32, 8>;
type SimdF32 = Simd<f32, 8>;
#[inline(always)]
fn cross_product_simd(
    ax: SimdF32, ay: SimdF32,
    bx: SimdF32, by: SimdF32,
    cx: SimdF32, cy: SimdF32,
    dx: SimdF32, dy: SimdF32,
) -> SimdF32 {
    let ab_x = bx - ax;
    let ab_y = by - ay;
    let cd_x = dx - cx;
    let cd_y = dy - cy;
    return ab_x * cd_y - ab_y * cd_x
}

#[inline(always)]
fn segments_intersect_simd(
    ax: SimdF32, ay: SimdF32, bx: SimdF32, by: SimdF32,
    cx: SimdF32, cy: SimdF32, dx: SimdF32, dy: SimdF32,
) -> SimdBool {
    let cp1 = cross_product_simd(ax, ay, bx, by, ax, ay, cx, cy);
    let cp2 = cross_product_simd(ax, ay, bx, by, ax, ay, dx, dy);
    let cp3 = cross_product_simd(cx, cy, dx, dy, cx, cy, ax, ay);
    let cp4 = cross_product_simd(cx, cy, dx, dy, cx, cy, bx, by);

    let zero = SimdF32::splat(0.0);
    
    // Check if segments straddle each other
    let straddle1 = (cp1.simd_gt(zero) & cp2.simd_lt(zero)) | (cp1.simd_lt(zero) & cp2.simd_gt(zero));
    let straddle2 = (cp3.simd_gt(zero) & cp4.simd_lt(zero)) | (cp3.simd_lt(zero) & cp4.simd_gt(zero));
    
    return straddle1 & straddle2
}

pub fn eliminate_crossings(tour: &mut Vec<shared::Point>) -> bool {
    let n = tour.len();
    if n < 4 {
        return false;
    }
    
    let mut improved = false;
    
    // Check all pairs of edges for crossings
    for i in 0..n {
        let next_i = (i + 1) % n;
        
        // Get coordinates for edge i
        let edge1_ax = tour[i].x;
        let edge1_ay = tour[i].y;
        let edge1_bx = tour[next_i].x;
        let edge1_by = tour[next_i].y;
        
        // Process edges in chunks of 8 using SIMD
        for chunk_start in ((i + 2)..n).step_by(8) {
            let mut ax_arr = [0.0f32; 8];
            let mut ay_arr = [0.0f32; 8];
            let mut bx_arr = [0.0f32; 8];
            let mut by_arr = [0.0f32; 8];
            let mut valid_mask = [false; 8];
            let mut edge_indices = [0usize; 8];
            
            // Fill SIMD arrays with up to 8 edges
            let mut count = 0;
            for j in chunk_start..n.min(chunk_start + 8) {
                // Skip adjacent edges (can't cross with immediate neighbors)
                if j == i || j == next_i || (j + 1) % n == i {
                    continue;
                }
                
                let next_j = (j + 1) % n;
                ax_arr[count] = tour[j].x;
                ay_arr[count] = tour[j].y;
                bx_arr[count] = tour[next_j].x;
                by_arr[count] = tour[next_j].y;
                valid_mask[count] = true;
                edge_indices[count] = j;
                count += 1;
            }
            
            if count == 0 {
                continue;
            }
            
            // Load into SIMD vectors
            let ax = SimdF32::from_array(ax_arr);
            let ay = SimdF32::from_array(ay_arr);
            let bx = SimdF32::from_array(bx_arr);
            let by = SimdF32::from_array(by_arr);
            
            let edge1_ax_simd = SimdF32::splat(edge1_ax);
            let edge1_ay_simd = SimdF32::splat(edge1_ay);
            let edge1_bx_simd = SimdF32::splat(edge1_bx);
            let edge1_by_simd = SimdF32::splat(edge1_by);
            
            // Check for intersections
            let intersections = segments_intersect_simd(
                edge1_ax_simd, edge1_ay_simd, edge1_bx_simd, edge1_by_simd,
                ax, ay, bx, by
            );
            
            // Process any intersections found
            for lane in 0..count {
                if valid_mask[lane] && intersections.test(lane) {
                    let j = edge_indices[lane];
                    
                    // Calculate improvement from uncrossing
                    let current_dist = math::calc_dist(tour[i], tour[next_i]) + 
                                     math::calc_dist(tour[j], tour[(j + 1) % n]);
                    
                    let uncrossed_dist = math::calc_dist(tour[i], tour[j]) + 
                                       math::calc_dist(tour[next_i], tour[(j + 1) % n]);
                    
                    if uncrossed_dist < current_dist {
                        // Perform 2-opt swap to uncross
                        let mut start = next_i;
                        let mut end = j;
                        
                        if start > end {
                            std::mem::swap(&mut start, &mut end);
                        }
                        
                        // Reverse the segment between the crossing edges
                        tour[start..=end].reverse();
                        improved = true;
                        
                        // Early termination after first improvement to avoid conflicts
                        return true;
                    }
                }
            }
        }
    }
    
    return improved
}

// Helper function to repeatedly eliminate crossings until no more are found
pub fn eliminate_all_crossings(tour: &mut Vec<shared::Point>) -> bool {
    let mut total_improved = false;
    
    while eliminate_crossings(tour) {
        total_improved = true;
    }
    
    return total_improved
}
