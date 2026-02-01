use std::hash::{Hash, Hasher};
use std::simd::Simd;

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

// Implement Hash for Point so it can be used in HashSet
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Convert f32 to bits for consistent hashing
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

// Implement Eq (required when implementing Hash with PartialEq)
impl Eq for Point {}

pub type SimdF32 = Simd<f32, 8>;
