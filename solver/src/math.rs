use crate::shared::{self};
use std::simd::{Simd, StdFloat};
use std::simd::num::SimdFloat;
use std::simd::cmp::SimdPartialOrd;

type SimdF32 = Simd<f32, 8>;

#[inline(always)]
pub fn calc_dist(a: shared::Point, b: shared::Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    // Fixed: dx² + dy² instead of dx² + dy * dy
    (dx.mul_add(dx, dy * dy)).sqrt()
}

#[inline(always)]
pub fn calc_dist_simd(a_x: shared::SimdF32, a_y: shared::SimdF32, b_x: shared::SimdF32, b_y: shared::SimdF32) -> shared::SimdF32 {
    let dx = a_x - b_x;
    let dy = a_y - b_y;
    // Fixed: dx² + dy² instead of dx² + dy * dy
    (dx.mul_add(dx, dy * dy)).sqrt()
}

#[inline(always)]
pub fn point_line(
    a_x: SimdF32, a_y: SimdF32,
    b_x: SimdF32, b_y: SimdF32,
    c_x: SimdF32, c_y: SimdF32,
) -> SimdF32 {
    let dx = b_x - a_x;
    let dy = b_y - a_y;
    // Fixed: dx² + dy² instead of dx² + dy * dy
    let line_length_squared = dx.mul_add(dx, dy * dy);

    let zero = SimdF32::splat(0.0);
    let one = SimdF32::splat(1.0);

    // Fixed: dot product calculation
    let t = ((c_x - a_x) * dx + (c_y - a_y) * dy) / line_length_squared;
    let t = t.simd_clamp(zero, one);

    let proj_x = t.mul_add(dx, a_x);
    let proj_y = t.mul_add(dy, a_y);
    let diff_x = c_x - proj_x;
    let diff_y = c_y - proj_y;

    (diff_x.mul_add(diff_x, diff_y * diff_y)).sqrt()
}

#[inline(never)]
pub fn fast_acos(x: SimdF32) -> SimdF32 {
    let x_abs = x.simd_max(-x);  
    let sqrt_term = (SimdF32::splat(1.0) - x_abs).sqrt();
    let base = SimdF32::splat(-0.0187293)
        .mul_add(x_abs, SimdF32::splat(0.0742610))
        .mul_add(x_abs, SimdF32::splat(-0.2121144))
        .mul_add(x_abs, SimdF32::splat(1.5707288));
    let ret = base * sqrt_term;

    let pi = SimdF32::splat(std::f32::consts::PI);
    let mask = x.simd_lt(SimdF32::splat(0.0));
    mask.select(pi - ret, ret)
}

#[inline(never)]
pub fn lda(
    a_x: SimdF32, a_y: SimdF32,
    b_x: SimdF32, b_y: SimdF32,
    c_x: SimdF32, c_y: SimdF32,
) -> SimdF32 {
    let ab = calc_dist_simd(a_x, a_y, b_x, b_y);
    let bc = calc_dist_simd(b_x, b_y, c_x, c_y);
    let ac = calc_dist_simd(a_x, a_y, c_x, c_y);

    // Fixed: cosine law calculation (b² + c² - a²) / (2bc)
    let numerator = bc.mul_add(bc, ac * ac) - ab * ab;
    let denominator = SimdF32::splat(2.0) * bc * ac;
    let cosine = numerator / denominator;

    // Clamp cosine to [-1, 1] to avoid numerical issues
    let cosine = cosine.simd_clamp(SimdF32::splat(-1.0), SimdF32::splat(1.0));

    // Use SIMD fast_acos approximation
    let acos = fast_acos(cosine);

    let dist = point_line(a_x, a_y, b_x, b_y, c_x, c_y);
    
    // Add small epsilon to avoid division by zero
    let epsilon = SimdF32::splat(1e-10);
    let safe_dist = dist.simd_max(epsilon);
    
    acos / safe_dist
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

    // Sort points lexicographically (x, then y)
    points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x).unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap())
    });

    let mut lower = Vec::new();
    for &p in &points {
        while lower.len() >= 2 {
            let l = lower.len();
            if shared::Point::cross(&lower[l - 2], &lower[l - 1], &p) <= 0.0 {
                lower.pop();
            } else {
                break;
            }
        }
        lower.push(p);
    }

    let mut upper = Vec::new();
    for &p in points.iter().rev() {
        while upper.len() >= 2 {
            let l = upper.len();
            if shared::Point::cross(&upper[l - 2], &upper[l - 1], &p) <= 0.0 {
                upper.pop();
            } else {
                break;
            }
        }
        upper.push(p);
    }

    // Remove duplicate endpoints
    lower.pop();
    upper.pop();

    lower.extend(upper);
    return lower
}