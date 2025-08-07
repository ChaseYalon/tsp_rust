use std::simd::{Simd};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn cross(o: &Point, a: &Point, b: &Point) -> f32 {
        return (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    }
}
pub type SimdF32 = Simd<f32, 8>;
