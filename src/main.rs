#![feature(portable_simd)]

use std::time::Instant;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::simd::{Simd, StdFloat};
use std::simd::num::SimdFloat;
use std::simd::prelude::SimdPartialOrd;
mod reader;
mod shared;

type SimdF32 = Simd<f32, 8>;

#[derive(Debug)]
struct InsertPointResult {
    lda: f32,
    best_a: shared::Point,
    best_c: shared::Point,
}
#[inline(always)]
fn calc_dist(a: shared::Point, b: shared::Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn calc_dist_simd(a_x: SimdF32, a_y: SimdF32, b_x: SimdF32, b_y: SimdF32) -> SimdF32 {
    let dx = a_x - b_x;
    let dy = a_y - b_y;
    (dx * dx + dy * dy).sqrt()
}

fn convex_hull(points: &[shared::Point]) -> Vec<shared::Point> {
    let mut points = points.to_vec();

    if points.len() <= 1 {
        return points;
    }

    points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x).unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap())
    });

    let mut lower = Vec::new();
    for p in &points {
        while lower.len() >= 2
            && shared::Point::cross(&lower[lower.len() - 2], &lower[lower.len() - 1], p) <= 0.0
        {
            lower.pop();
        }
        lower.push(*p);
    }

    let mut upper = Vec::new();
    for p in points.iter().rev() {
        while upper.len() >= 2
            && shared::Point::cross(&upper[upper.len() - 2], &upper[upper.len() - 1], p) <= 0.0
        {
            upper.pop();
        }
        upper.push(*p);
    }

    lower.pop();
    upper.pop();

    lower.extend(upper);
    lower
}

#[inline(always)]
fn point_line_simd(
    a_x: SimdF32, a_y: SimdF32,
    b_x: SimdF32, b_y: SimdF32,
    c_x: SimdF32, c_y: SimdF32,
) -> SimdF32 {
    let dx = b_x - a_x;
    let dy = b_y - a_y;
    let line_length_squared = dx * dx + dy * dy;

    let zero = SimdF32::splat(0.0);
    let one = SimdF32::splat(1.0);

    let t = ((c_x - a_x) * dx + (c_y - a_y) * dy) / line_length_squared;
    let t = t.simd_clamp(zero, one);

    let proj_x = a_x + t * dx;
    let proj_y = a_y + t * dy;
    let diff_x = c_x - proj_x;
    let diff_y = c_y - proj_y;

    (diff_x * diff_x + diff_y * diff_y).sqrt()
}

#[inline(always)]
fn fast_acos_simd(x: SimdF32) -> SimdF32 {
    let x_bits = x.to_bits();
    let masked_bits = x_bits & Simd::splat(0x7FFF_FFFF);
    let x_abs = SimdF32::from_bits(masked_bits);

    let sqrt_term = (SimdF32::splat(1.0) - x_abs).sqrt();
    let base = ((SimdF32::splat(-0.0187293) * x_abs + SimdF32::splat(0.0742610)) * x_abs
                - SimdF32::splat(0.2121144)) * x_abs + SimdF32::splat(1.5707288);
    let ret = base * sqrt_term;

    let pi = SimdF32::splat(std::f32::consts::PI);
    let mask = x.simd_lt(SimdF32::splat(0.0));
    mask.select(pi - ret, ret)
}

fn lda(
    a_x: SimdF32, a_y: SimdF32,
    b_x: SimdF32, b_y: SimdF32,
    c_x: SimdF32, c_y: SimdF32,
) -> SimdF32 {
    let ab = calc_dist_simd(a_x, a_y, b_x, b_y);
    let bc = calc_dist_simd(b_x, b_y, c_x, c_y);
    let ac = calc_dist_simd(a_x, a_y, c_x, c_y);

    let numerator = ac * ac + bc * bc - ab * ab;
    let denominator = SimdF32::splat(2.0) * bc * ac;
    let cosine = numerator / denominator;

    // Use SIMD fast_acos approximation
    let acos = fast_acos_simd(cosine);

    let dist = point_line_simd(a_x, a_y, b_x, b_y, c_x, c_y);
    acos / dist
}

fn insert_point(hull: &[shared::Point], inner_points: &[shared::Point]) -> InsertPointResult {
    let hull_len = hull.len();
    let hull_x: Vec<f32> = hull.iter().map(|p| p.x).collect();
    let hull_y: Vec<f32> = hull.iter().map(|p| p.y).collect();

    let chunk_size = 8;

    inner_points.par_chunks(chunk_size).map(|chunk| {
        let len = chunk.len();
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

        for j in 0..(hull_len - 1) {
            let a_x = SimdF32::splat(hull_x[j]);
            let a_y = SimdF32::splat(hull_y[j]);
            let b_x = SimdF32::splat(hull_x[j + 1]);
            let b_y = SimdF32::splat(hull_y[j + 1]);

            let curr_lda = lda(a_x, a_y, b_x, b_y, c_x, c_y);

            let mask = curr_lda.simd_gt(best_lda);
            best_lda = mask.select(curr_lda, best_lda);
            best_idx = mask.select(Simd::splat(j as i32), best_idx);

        }

        // Wraparound edge
        let j = hull_len - 1;
        let a_x = SimdF32::splat(hull_x[j]);
        let a_y = SimdF32::splat(hull_y[j]);
        let b_x = SimdF32::splat(hull_x[0]);
        let b_y = SimdF32::splat(hull_y[0]);

        let curr_lda = lda(a_x, a_y, b_x, b_y, c_x, c_y);
        let mask = curr_lda.simd_gt(best_lda);
        if mask.any() {
            best_lda = mask.select(curr_lda, best_lda);
            best_idx = mask.select(Simd::splat(j as i32), best_idx);
        }

        let mut best_lane = 0;
        let mut best_lda_scalar = best_lda[0];
        for lane in 1..len {
            if best_lda[lane] > best_lda_scalar {
                best_lda_scalar = best_lda[lane];
                best_lane = lane;
            }
        }

        let idx = best_idx[best_lane] as usize;
        let next_idx = (idx + 1) % hull_len;

        InsertPointResult {
            lda: best_lda_scalar,
            best_a: shared::Point {
                x: hull_x[idx],
                y: hull_y[idx],
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

fn path_dist(path: &[shared::Point]) -> f32 {
    let mut sum = 0.0;
    for i in 0..path.len() - 1 {
        sum += calc_dist(path[i], path[i + 1]);
    }
    sum += calc_dist(path[path.len() - 1], path[0]);
    sum
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(20).build_global().unwrap();

    let points: Vec<shared::Point> = reader::parse_file(&reader::read_file());

    let start = Instant::now();

    let mut hull = convex_hull(&points);
    let mut inner_hull = reader::vec_diff(&points, &hull);

    let pb = ProgressBar::new(inner_hull.len().try_into().unwrap());

    while !inner_hull.is_empty() {
        let result = insert_point(&hull, &inner_hull);
        update_hull(&result, &mut hull, &mut inner_hull);
        pb.inc(1);
    }

    pb.finish();
    let dist = path_dist(&hull);
    let elapsed = start.elapsed();

    println!("{:?}", dist);
    println!("Elapsed: {:.2?}", elapsed);
}
