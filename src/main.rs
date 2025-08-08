#![feature(portable_simd)]
#![feature(duration_millis_float)]
use std::time::Instant;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::simd::{Simd};
use std::simd::prelude::SimdPartialOrd;

use crate::reader::{should_edge_swap, should_log, should_or_opt, should_relp};
use crate::relp::{remove_points_from_hull, find_lowest_lda_points};

mod relp;
mod reader;
mod shared;
mod math;
mod edges;
mod or_opt;
type SimdF32 = Simd<f32, 8>;




fn insert_point(hull: &[shared::Point], inner_points: &[shared::Point]) -> relp::InsertPointResult {
    let hull_len = hull.len();

    // Precompute and cache A–B segments (wraparound included)
    let mut a_x_cache = Vec::with_capacity(hull_len);
    let mut a_y_cache = Vec::with_capacity(hull_len);
    let mut b_x_cache = Vec::with_capacity(hull_len);
    let mut b_y_cache = Vec::with_capacity(hull_len);

    for j in 0..hull_len {
        let a = hull[j];
        let b = hull[(j + 1) % hull_len];
        a_x_cache.push(a.x);
        a_y_cache.push(a.y);
        b_x_cache.push(b.x);
        b_y_cache.push(b.y);
    }

    let chunk_size = 8;

    inner_points.par_chunks(chunk_size).map(|chunk| {
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

        let mut best_lda = SimdF32::splat(-1.0);
        let mut best_idx = Simd::<i32, 8>::splat(-1);

        for j in 0..hull_len {
            let a_x = SimdF32::splat(a_x_cache[j]);
            let a_y = SimdF32::splat(a_y_cache[j]);
            let b_x = SimdF32::splat(b_x_cache[j]);
            let b_y = SimdF32::splat(b_y_cache[j]);

            let curr_lda = math::lda(a_x, a_y, b_x, b_y, c_x, c_y);

            let mask = curr_lda.simd_gt(best_lda);
            best_lda = mask.select(curr_lda, best_lda);
            best_idx = mask.select(Simd::splat(j as i32), best_idx);
        }

        // Pick best result in SIMD lane
        let mut best_lane = 0;
        let mut best_lda_scalar = best_lda[0];
        for lane in 1..len {
            if best_lda[lane] > best_lda_scalar {
                best_lda_scalar = best_lda[lane];
                best_lane = lane;
            }
        }

        let idx = best_idx[best_lane] as usize;

        relp::InsertPointResult {
            lda: best_lda_scalar,
            best_a: shared::Point {
                x: a_x_cache[idx],
                y: a_y_cache[idx],
            },
            best_c: shared::Point {
                x: c_x[best_lane],
                y: c_y[best_lane],
            },
        }
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


fn main() {
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    let points: Vec<shared::Point> = reader::parse_file(&reader::read_file());

    let start = Instant::now();

    let mut hull = math::convex_hull(&points);
    let mut inner_hull = reader::vec_diff(&points, &hull);
    let mut insert_log: Vec<relp::InsertPointResult> = Vec::with_capacity(inner_hull.len());

    let sl = should_log();
    let mut pb: ProgressBar = ProgressBar::new(1);
    if !sl{
        println!("Logging disabled");
    }
    if sl {
        pb = ProgressBar::new(inner_hull.len().try_into().unwrap());
    }
    while !inner_hull.is_empty() {
        let result = insert_point(&hull, &inner_hull);
        update_hull(&result, &mut hull, &mut inner_hull, &mut insert_log);
        if sl{
            pb.inc(1);
        }
    }
    if sl{
        pb.finish();

    }
    let dist = math::path_dist(&hull);
    let elapsed = start.elapsed();

    println!("{:?}", dist);
    println!("Elapsed: {:.2?}", elapsed);
    let o_start = Instant::now();
    /*
        This is the post processing section
        The idea is to take quick easy fixes to "smooth out" some of the rough spots
        The user can disable each section with a terminal command
        Should add "global kill" for disabling all post processing
     */
    //0.03% ~2x execution time per 200 long edges tested
    if should_edge_swap(){
        edges::eliminate_all_crossings(&mut hull);
    }
    //0.01% ~0.02 secs
    if should_or_opt(){
        or_opt::multi_or_opt_optimization(&mut hull);
    }
    //0.025 ~5 secs
    if should_relp(){
        let mut new_inner_hull = find_lowest_lda_points(&insert_log, hull.len() / 4);
        remove_points_from_hull(&mut hull, &new_inner_hull);
        while !new_inner_hull.is_empty() {
            let result = insert_point(&hull, &new_inner_hull);
            update_hull(&result, &mut hull, &mut new_inner_hull, &mut insert_log);
        }
    }

    let o_end = o_start.elapsed().as_millis_f32();
    let new_dist = math::path_dist(&hull);
    println!("Improved the tour to dist of {:.2?} with a {:.2?}% improvement using {:.2?} seconds", new_dist, (dist / new_dist) - 1.0, o_end / 1000.0);
}
