#![feature(portable_simd)]
#![feature(duration_millis_float)]
use std::time::Instant;
use indicatif::ProgressBar;
use kdtree::KdTree;
use rayon::prelude::*;
use std::simd::{Simd};
use std::simd::prelude::SimdPartialOrd;
use std::env;
use std::fs;

use crate::math::path_dist;
use crate::precompute::calc_kd_tree;
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

fn insert_point(
    hull: &[shared::Point], 
    inner_points: &[shared::Point], 
    kdtree: &KdTree<f32, shared::Point, [f32; 2]>,
    n: usize
) -> relp::InsertPointResult {
    let hull_len = hull.len();
    
    // Build HashSet once for fast inner_points lookup
    let inner_points_set: std::collections::HashSet<shared::Point> = 
        inner_points.iter().copied().collect();
    
    // Process hull edges in parallel
    (0..hull_len).into_par_iter().map(|j| {
        let a = hull[j];
        let b = hull[(j + 1) % hull_len];
        
        // Use edge midpoint as query point for kdtree
        let query_point = shared::Point {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        };
        
        // Get candidates from the main kdtree
        let closest_points = precompute::nearest_k_points(kdtree, query_point, n);
        
        // Filter to only valid candidates using O(1) HashSet lookup
        let mut valid_candidates = Vec::with_capacity(n.min(closest_points.len()));
        for point in closest_points {
            if inner_points_set.contains(&point) {
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
    insert_log: &mut Vec<relp::InsertPointResult>,
) {
    // Log this insertion for potential post-processing
    insert_log.push(result.clone());

    // Remove the inserted point from the inner point set
    if let Some(pos) = inner_hull.iter().position(|&p| p == result.best_c) {
        inner_hull.remove(pos);
    }

    // Insert the point after best_a in the hull
    if let Some(pos) = hull.iter().position(|&p| p == result.best_a) {
        hull.insert(pos + 1, result.best_c);
    }
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

    let start = Instant::now();
    let kdtree = calc_kd_tree(&points);
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

    let sl = should_log();
    let mut pb: ProgressBar = ProgressBar::new(1);
    if sl {
        println!("Logging disabled");
    }
    if !sl {
        pb = ProgressBar::new(inner_hull.len().try_into().unwrap());
    }
    
    // Use adaptive n based on problem size
    let adaptive_n = (64_usize).min(inner_hull.len() / 10).max(8);
    
    while !inner_hull.is_empty() {
        let result = insert_point(&hull, &inner_hull, &kdtree, adaptive_n);
        update_hull(&result, &mut hull, &mut inner_hull, &mut insert_log);
        if !sl {
            pb.inc(1);
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
        let mut new_inner_hull = find_lowest_lda_points(&insert_log, hull.len() / 8);
        remove_points_from_hull(&mut hull, &new_inner_hull);
        while !new_inner_hull.is_empty() {
            let result = insert_point(&hull, &new_inner_hull, &kdtree, adaptive_n);
            update_hull(&result, &mut hull, &mut new_inner_hull, &mut insert_log);
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
    
    // Always write to the consistent output path
    write_to_tsp_file(&hull, &output_path);
}