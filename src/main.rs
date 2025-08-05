#![feature(portable_simd)]

use std::time::Instant;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::simd::{Simd, StdFloat};
use std::simd::num::SimdFloat;
use std::simd::cmp::SimdPartialOrd;

mod reader;
mod shared;

#[derive(Debug)]
struct InsertPointResult {
    lda: f32,
    
    best_a: shared::Point,
    best_c: shared::Point,
}

fn calc_dist(a: shared::Point, b: shared::Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn calc_dist_simd(a_x: Simd<f32, 4>, a_y: Simd<f32, 4>, b_x: Simd<f32, 4>, b_y: Simd<f32, 4>) -> Simd<f32, 4> {
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

fn point_line(a: &shared::Point, b: &shared::Point, c: &shared::Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let line_length_squared = dx * dx + dy * dy;

    if line_length_squared == 0.0 {
        let dx = c.x - a.x;
        let dy = c.y - a.y;
        return (dx * dx + dy * dy).sqrt();
    }

    let mut t = ((c.x - a.x) * dx + (c.y - a.y) * dy) / line_length_squared;
    t = t.clamp(0.0, 1.0);

    let proj_x = a.x + t * dx;
    let proj_y = a.y + t * dy;
    let diff_x = c.x - proj_x;
    let diff_y = c.y - proj_y;

    (diff_x * diff_x + diff_y * diff_y).sqrt()
}

fn point_line_simd(
    a_x: Simd<f32, 4>, a_y: Simd<f32, 4>,
    b_x: Simd<f32, 4>, b_y: Simd<f32, 4>,
    c_x: Simd<f32, 4>, c_y: Simd<f32, 4>,
) -> Simd<f32, 4> {
    let dx = b_x - a_x;
    let dy = b_y - a_y;
    let line_length_squared = dx * dx + dy * dy;

    let zero = Simd::splat(0.0);
    let one = Simd::splat(1.0);

    let t = ((c_x - a_x) * dx + (c_y - a_y) * dy) / line_length_squared;
    let t = t.simd_clamp(zero, one);

    let proj_x = a_x + t * dx;
    let proj_y = a_y + t * dy;
    let diff_x = c_x - proj_x;
    let diff_y = c_y - proj_y;

    (diff_x * diff_x + diff_y * diff_y).sqrt()
}

// Much faster approximation
#[inline(always)]
fn fast_acos(x: f32) -> f32 {
    let x_abs = f32::from_bits(x.to_bits() & 0x7FFF_FFFF);
    let sqrt_term = (1.0 - x_abs).sqrt();
    let base = ((-0.0187293 * x_abs + 0.0742610) * x_abs - 0.2121144) * x_abs + 1.5707288;
    let ret = base * sqrt_term;

    if x.is_sign_negative() {
        std::f32::consts::PI - ret
    } else {
        ret
    }
}

fn fast_acos_simd(x: Simd<f32, 4>) -> Simd<f32, 4> {
    // Convert f32 lanes to u32 lanes (bitwise)
    let x_bits: Simd<u32, 4> = x.to_bits();
    // Mask out the sign bit (0x7FFF_FFFF)
    let masked_bits = x_bits & Simd::splat(0x7FFF_FFFF);
    // Convert back to f32 lanes
    let x_abs = Simd::<f32, 4>::from_bits(masked_bits);

    let sqrt_term = (Simd::splat(1.0) - x_abs).sqrt();
    let base = ((Simd::splat(-0.0187293) * x_abs + Simd::splat(0.0742610)) * x_abs
                - Simd::splat(0.2121144)) * x_abs + Simd::splat(1.5707288);
    let ret = base * sqrt_term;

    let pi = Simd::splat(std::f32::consts::PI);
    // If lane is negative, do PI - ret else ret
    let mask = x.simd_lt(Simd::splat(0.0));
    return mask.select(pi - ret, ret);
}


fn lda(
    a_x: Simd<f32, 4>, a_y: Simd<f32, 4>,
    b_x: Simd<f32, 4>, b_y: Simd<f32, 4>,
    c_x: Simd<f32, 4>, c_y: Simd<f32, 4>,
) -> Simd<f32, 4> {
    let ab = calc_dist_simd(a_x, a_y, b_x, b_y);
    let bc = calc_dist_simd(b_x, b_y, c_x, c_y);
    let ac = calc_dist_simd(a_x, a_y, c_x, c_y);

    let numerator = ac * ac + bc * bc - ab * ab;
    let denominator = Simd::splat(2.0) * bc * ac;
    let cosine = numerator / denominator;

    let acos = {
        // Approximate fast_acos SIMD: use scalar fallback or a simple vectorized approximation
        // Here for brevity, fallback to scalar per lane:
        let mut result = Simd::splat(0.0);
        for i in 0..4 {
            result[i] = fast_acos(cosine[i]);
        }
        result
    };

    let dist = point_line_simd(a_x, a_y, b_x, b_y, c_x, c_y);
    acos / dist
}

fn insert_point(hull: &[shared::Point], inner_points: &[shared::Point]) -> InsertPointResult {
    let hull_len = hull.len();
    let hull_x: Vec<f32> = hull.iter().map(|p| p.x).collect();
    let hull_y: Vec<f32> = hull.iter().map(|p| p.y).collect();

    let chunk_size = 4;

    // We parallelize outer loop with rayon
    inner_points.par_chunks(chunk_size).map(|chunk| {
        let len = chunk.len();
        let mut c_x_arr = [0.0; 4];
        let mut c_y_arr = [0.0; 4];
        for i in 0..len {
            c_x_arr[i] = chunk[i].x;
            c_y_arr[i] = chunk[i].y;
        }
        let c_x = Simd::from_array(c_x_arr);
        let c_y = Simd::from_array(c_y_arr);

        let mut best_lda = Simd::splat(-1.0);
        let mut best_a_x = Simd::splat(0.0);
        let mut best_a_y = Simd::splat(0.0);
        let mut best_c_x = Simd::splat(0.0);
        let mut best_c_y = Simd::splat(0.0);

        for j in 0..(hull_len - 1) {
            let a_x = Simd::splat(hull_x[j]);
            let a_y = Simd::splat(hull_y[j]);
            let b_x = Simd::splat(hull_x[j + 1]);
            let b_y = Simd::splat(hull_y[j + 1]);

            let curr_lda = lda(a_x, a_y, b_x, b_y, c_x, c_y);

            let mask = curr_lda.simd_gt(best_lda);
            best_lda = mask.select(curr_lda, best_lda);
            best_a_x = mask.select(a_x, best_a_x);
            best_a_y = mask.select(a_y, best_a_y);
            best_c_x = mask.select(c_x, best_c_x);
            best_c_y = mask.select(c_y, best_c_y);
        }

        // Wraparound edge
        let a_x = Simd::splat(hull_x[hull_len - 1]);
        let a_y = Simd::splat(hull_y[hull_len - 1]);
        let b_x = Simd::splat(hull_x[0]);
        let b_y = Simd::splat(hull_y[0]);

        let curr_lda = lda(a_x, a_y, b_x, b_y, c_x, c_y);
        let mask = curr_lda.simd_gt(best_lda);
        best_lda = mask.select(curr_lda, best_lda);
        best_a_x = mask.select(a_x, best_a_x);
        best_a_y = mask.select(a_y, best_a_y);
        best_c_x = mask.select(c_x, best_c_x);
        best_c_y = mask.select(c_y, best_c_y);

        // Find best lane in this chunk
        let mut best_lane = 0;
        let mut best_lda_scalar = best_lda[0];
        for lane in 1..len {
            if best_lda[lane] > best_lda_scalar {
                best_lda_scalar = best_lda[lane];
                best_lane = lane;
            }
        }

        InsertPointResult {
            lda: best_lda_scalar,
            best_a: shared::Point {
                x: best_a_x[best_lane],
                y: best_a_y[best_lane],
            },
            best_c: shared::Point {
                x: best_c_x[best_lane],
                y: best_c_y[best_lane],
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
