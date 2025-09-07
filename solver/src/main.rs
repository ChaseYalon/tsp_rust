#![feature(portable_simd)]
#![feature(duration_millis_float)]
use std::time::{Duration, Instant};
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::simd::{Simd};
use std::env;
use std::fs;
use std::sync::{Arc, Mutex};

use std::io::Write;

use crate::math::path_dist;
use crate::precompute::{SpatialGrid, calculate_search_radius};
use crate::reader::{no_post, should_edge_swap, should_log, should_or_opt, should_relp, write_to_tsp_file};
use crate::relp::{remove_points_from_hull, find_lowest_lda_points};

mod relp;
mod reader;
mod shared;
mod math;
mod edges;
mod or_opt;
mod precompute;

type SimdF32 = Simd<f32, 8>;

#[inline(never)]
fn insert_point(
    hull: &[shared::Point], 
    spatial_grid: &SpatialGrid,
    n: usize,
) -> relp::InsertPointResult {
    let hull_len = hull.len();
    let search_radius = calculate_search_radius(hull);
    
    // Process hull edges in parallel
    (0..hull_len).into_par_iter().map(|j| {
        let a = hull[j];
        let b = hull[(j + 1) % hull_len];
        
        // Get candidates from spatial grid using edge-based query
        let candidates = spatial_grid.query_edge_candidates(a, b, search_radius);
        
        // Filter to only include points that are actually in the spatial grid
        // (i.e., haven't been removed yet - represents inner_points)
        let mut valid_candidates = Vec::with_capacity(candidates.len().min(n));
        for point in candidates {
            if spatial_grid.contains_point(point) {
                valid_candidates.push(point);
                // Early termination if we have enough candidates
                if valid_candidates.len() >= n.min(32) {
                    break;
                }
            }
        }
        
        if valid_candidates.is_empty() {
            return relp::InsertPointResult {
                lda: -1.0,
                best_a: shared::Point { x: 0.0, y: 0.0 },
                best_c: shared::Point { x: 0.0, y: 0.0 },
            };
        }
        
        // Process candidates in SIMD chunks
        let chunk_size = 8;
        let mut edge_best = relp::InsertPointResult {
            lda: -1.0,
            best_a: a,
            best_c: shared::Point { x: 0.0, y: 0.0 },
        };
        
        // Precompute edge constants for SIMD
        let a_x_simd = SimdF32::splat(a.x);
        let a_y_simd = SimdF32::splat(a.y);
        let b_x_simd = SimdF32::splat(b.x);
        let b_y_simd = SimdF32::splat(b.y);
        
        for chunk in valid_candidates.chunks(chunk_size) {
            let len = chunk.len();

            // Load chunk into SIMD vectors
            let mut c_x_arr = [0.0; 8];
            let mut c_y_arr = [0.0; 8];
            for i in 0..len {
                c_x_arr[i] = chunk[i].x;
                c_y_arr[i] = chunk[i].y;
            }
            let c_x = SimdF32::from_array(c_x_arr);
            let c_y = SimdF32::from_array(c_y_arr);

            // Calculate LDA for this edge with all candidates in the chunk
            let curr_lda = math::lda(a_x_simd, a_y_simd, b_x_simd, b_y_simd, c_x, c_y);

            // Find best in this chunk and update edge_best if better
            for lane in 0..len {
                if curr_lda[lane] > edge_best.lda {
                    edge_best.lda = curr_lda[lane];
                    edge_best.best_a = a;
                    edge_best.best_c = chunk[lane];
                }
            }
        }
        edge_best
    })
    .reduce(
        || relp::InsertPointResult {
            lda: -1.0,
            best_a: shared::Point { x: 0.0, y: 0.0 },
            best_c: shared::Point { x: 0.0, y: 0.0 },
        },
        |a, b| if a.lda > b.lda { a } else { b },
    )
}

fn update_hull(
    result: &relp::InsertPointResult,
    hull: &mut Vec<shared::Point>,
    inner_hull: &mut Vec<shared::Point>,
    spatial_grid: &mut SpatialGrid,
    insert_log: &mut Vec<relp::InsertPointResult>,
    mes: &Arc<Mutex<Vec<Duration>>>,
    durr: Duration
) {
    // Log this insertion for potential post-processing
    insert_log.push(result.clone());

    // Remove the inserted point from the inner point set
    if let Some(pos) = inner_hull.iter().position(|&p| p == result.best_c) {
        inner_hull.remove(pos);
    }
    
    // Remove from spatial grid
    spatial_grid.remove_point(result.best_c);

    // Insert the point after best_a in the hull
    if let Some(pos) = hull.iter().position(|&p| p == result.best_a) {
        hull.insert(pos + 1, result.best_c);
    }
    let mut mesa = mes.lock().unwrap();
    mesa.push(durr);
}

fn get_output_path() -> String {
    // Try to determine the correct output path
    if std::path::Path::new("backend/output").exists() {
        "backend/output/OUT.tsp".to_string()
    } else if std::path::Path::new("output").exists() {
        "output/OUT.tsp".to_string()
    } else {
        // Create the directory if it doesn't exist
        std::fs::create_dir_all("backend/output").unwrap_or_else(|_| {
            std::fs::create_dir_all("output").unwrap();
        });
        if std::path::Path::new("backend/output").exists() {
            "backend/output/OUT.tsp".to_string()
        } else {
            "output/OUT.tsp".to_string()
        }
    }
}
fn main() {
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    let points: Vec<shared::Point> = reader::parse_file(&reader::read_file());
    let mes_arr: Arc<Mutex<Vec<Duration>>> = Arc::new(Mutex::new(Vec::with_capacity(points.len())));
    let start = Instant::now();
    
    // Build spatial grid instead of kdtree
    let mut spatial_grid = SpatialGrid::new(&points);
    
    let mut hull = math::convex_hull(&points);
    let mut inner_hull = reader::vec_diff(&points, &hull);
    let mut insert_log: Vec<relp::InsertPointResult> = Vec::with_capacity(inner_hull.len());

    if hull.len() == 0 {
        eprintln!("Hull length is zero, input was not read properly, args are {:#?}", env::args().collect::<Vec<_>>());
        if let Some(arg) = env::args().nth(1) {
            eprintln!("File is {:?}", fs::read_to_string(arg));
        } else {
            eprintln!("No argument provided");
        }
        std::process::exit(1);
    }

    // Remove hull points from spatial grid since they're not "inner" points
    for &hull_point in &hull {
        spatial_grid.remove_point(hull_point);
    }

    let sl = should_log();
    let mut pb: ProgressBar = ProgressBar::new(1);
    if sl {
        println!("Logging disabled");
    }
    if !sl {
        pb = ProgressBar::new(inner_hull.len().try_into().unwrap());
    }
    
    let adaptive_n = (64_usize).min(inner_hull.len() / 10).max(8);
    
    let max_iterations = inner_hull.len() * 2; // Safety margin
    let mut iteration_count = 0;
    
    while !inner_hull.is_empty() && iteration_count < max_iterations {
        iteration_count += 1;
        let start = Instant::now();
        let result = insert_point(&hull, &spatial_grid, adaptive_n);
        let end = start.elapsed();

        // If no valid insertion found, try fallback strategy
        if result.lda <= 0.0 {
            
            // Find the closest point to any hull point as fallback
            let mut best_fallback = relp::InsertPointResult {
                lda: 0.1, // Small positive value to ensure insertion
                best_a: hull[0],
                best_c: inner_hull[0],
            };
            
            let mut min_distance = f32::INFINITY;
            for &inner_point in &inner_hull {
                for (_i, &hull_point) in hull.iter().enumerate() {
                    let dx = inner_point.x - hull_point.x;
                    let dy = inner_point.y - hull_point.y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance < min_distance {
                        min_distance = distance;
                        best_fallback.best_a = hull_point;
                        best_fallback.best_c = inner_point;
                    }
                }
            }
            
            
            update_hull(&best_fallback, &mut hull, &mut inner_hull, &mut spatial_grid, &mut insert_log , &mes_arr, Instant::now().elapsed());
        } else {
            update_hull(&result, &mut hull, &mut inner_hull, &mut spatial_grid, &mut insert_log, &mes_arr, end);
        }
        
        if !sl {
            pb.inc(1);
        }
    }
    
    if iteration_count >= max_iterations {
        println!("Hit iteration limit, possible infinite loop detected. Remaining points: {}", inner_hull.len());
        // Force exit with remaining points
        for &remaining_point in &inner_hull {
            println!("Unprocessed point: ({}, {})", remaining_point.x, remaining_point.y);
        }
    }
    
    if !sl {
        pb.finish();
    }
    
    let dist = math::path_dist(&hull);
    let elapsed = start.elapsed();
    if !sl {
        println!("{:?}", dist);
        println!("Elapsed: {:.2?}", elapsed);
    }

    let o_start = Instant::now();
    
    // Get consistent output path
    let output_path = get_output_path();
    
    if no_post() {
        let mes_vec = mes_arr.lock().unwrap();
        let nanos: Vec<u64> = mes_vec.iter().map(|d| d.as_nanos() as u64).collect();
        
        use std::fs::OpenOptions;
        let mut file = OpenOptions::new()
            .create(true)    // Create if doesn't exist
            .append(true)    // Append to end of file
            .open("MEASUREMENT.txt")
            .expect("Failed to open file");
        
        // Write in human-readable format with timestamp
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        writeln!(file, "Run at timestamp {}: {:?}", timestamp, nanos).expect("Failed to write file");
        file.flush().expect("Failed to flush file");
        println!("File written successfully");

        println!("Operation completed, written to file");
        write_to_tsp_file(&hull, &output_path);
        std::process::exit(0);
    }
    
    if !should_edge_swap() {
        edges::eliminate_all_crossings(&mut hull);
    }
    if !should_or_opt() {
        or_opt::multi_or_opt_optimization(&mut hull);
    }
    if !should_relp() {
        // Rebuild spatial grid for re-optimization phase
        let mut new_inner_hull = find_lowest_lda_points(&insert_log, hull.len() / 8);
        let mut reopt_spatial_grid = SpatialGrid::new(&new_inner_hull);
        
        remove_points_from_hull(&mut hull, &new_inner_hull);
        
        while !new_inner_hull.is_empty() {
            let start = Instant::now();
            let result = insert_point(&hull, &reopt_spatial_grid, adaptive_n);
            let end = start.elapsed();
            update_hull(&result, &mut hull, &mut new_inner_hull, &mut reopt_spatial_grid, &mut insert_log, &mes_arr, end);
        }
    }
    println!("{:.2?}", path_dist(&hull));

    let o_end = o_start.elapsed().as_millis_f32();
    let new_dist = math::path_dist(&hull);
    if !sl {
        println!("Improved the tour to dist of {:.2?} with a {:.2?}% improvement using {:.2?} seconds", new_dist, (dist / new_dist) - 1.0, o_end / 1000.0);
    } else {
        println!("Operation completed, written to file");
    }
    let mes_vec = mes_arr.lock().unwrap();
    let nanos: Vec<u64> = mes_vec.iter().map(|d| d.as_nanos() as u64).collect();
    
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new()
        .create(true)    // Create if doesn't exist
        .append(true)    // Append to end of file
        .open("MEASUREMENT.txt")
        .expect("Failed to open file");
    
    // Write in human-readable format with timestamp
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    writeln!(file, "Run at timestamp {}: {:?}", timestamp, nanos).expect("Failed to write file");
    file.flush().expect("Failed to flush file");
    println!("File written successfully");

    write_to_tsp_file(&hull, &output_path);
}