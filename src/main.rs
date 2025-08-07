#![feature(portable_simd)]

use std::time::Instant;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::simd::{Simd};
use std::simd::prelude::SimdPartialOrd;

use crate::reader::should_log;
mod reader;
mod shared;
mod math;
type SimdF32 = Simd<f32, 8>;

#[derive(Debug)]
struct InsertPointResult {
    lda: f32,
    best_a: shared::Point,
    best_c: shared::Point,
}


fn insert_point(hull: &[shared::Point], inner_points: &[shared::Point]) -> InsertPointResult {
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

        InsertPointResult {
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
        || InsertPointResult {
            lda: -1.0,
            best_a: shared::Point { x: 0.0, y: 0.0 },
            best_c: shared::Point { x: 0.0, y: 0.0 },
        },
        |a, b| if a.lda > b.lda { a } else { b },
    )
}


fn update_hull(
    result: &InsertPointResult,
    hull: &mut Vec<shared::Point>,
    inner_hull: &mut Vec<shared::Point>,
) {
    if let Some(pos) = inner_hull.iter().position(|&p| p == result.best_c) {
        inner_hull.remove(pos);
    }

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
        update_hull(&result, &mut hull, &mut inner_hull);
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
}
