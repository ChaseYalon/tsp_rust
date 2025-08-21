use crate::shared::{Point};
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;

pub fn calc_kd_tree(points: &Vec<Point>) -> KdTree<f32, Point, [f32; 2]> {
    let mut kdtree = KdTree::new(2); // 2D points
    
    for point in points.iter() {
        kdtree.add([point.x, point.y], *point).unwrap();
    }
    kdtree
}

pub fn nearest_k_points(kdtree: &KdTree<f32, Point, [f32; 2]>, query_point: Point, k: usize) -> Vec<Point> {
    let nearest = kdtree.nearest(&[query_point.x, query_point.y], k, &squared_euclidean).unwrap();
    
    // Extract just the Point values
    let nearest_points: Vec<Point> = nearest.iter().map(|(_, p)| **p).collect();
    return nearest_points
}