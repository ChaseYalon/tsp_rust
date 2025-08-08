use crate::shared::{self};
use std::simd::{Simd, StdFloat};
use std::simd::num::SimdFloat;
use std::simd::cmp::SimdPartialOrd;

type SimdF32 = Simd<f32, 8>;

#[inline(always)]
pub fn calc_dist(a: shared::Point, b: shared::Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}
#[inline(always)]
pub fn calc_dist_simd(a_x: shared::SimdF32, a_y: shared::SimdF32, b_x: shared::SimdF32, b_y: shared::SimdF32) -> shared::SimdF32 {
    let dx = a_x - b_x;
    let dy = a_y - b_y;
    (dx * dx + dy * dy).sqrt()
}
#[inline(always)]
pub fn point_line(
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
pub fn fast_acos(x: SimdF32) -> SimdF32 {
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

#[inline(always)]
pub fn lda(
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
    let acos = fast_acos(cosine);

    let dist = point_line(a_x, a_y, b_x, b_y, c_x, c_y);
    return acos / dist
}

pub fn path_dist(path: &[shared::Point]) -> f32 {
    let mut sum = 0.0;
    for i in 0..path.len() - 1 {
        sum += calc_dist(path[i], path[i + 1]);
    }
    sum += calc_dist(path[path.len() - 1], path[0]);
    sum
}
pub fn convex_hull(points: &[shared::Point]) -> Vec<shared::Point> {
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

