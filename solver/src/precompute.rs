use crate::shared::Point;
// use std::collections::{HashMap,HashSet};
use rustc_hash::{ FxHashMap as HashMap };
pub struct SpatialGrid {
    grid: HashMap<(i32, i32), Vec<Point>>,
    cell_size: f32,
    min_x: f32,
    min_y: f32,
}

impl SpatialGrid {
    pub fn new(points: &[Point]) -> Self {
        // Calculate bounds
        let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
        
        // Calculate average edge length for cell sizing
        let width = max_x - min_x;
        let height = max_y - min_y;
        let diagonal = (width * width + height * height).sqrt();
        
        // Cell size should be roughly 1/sqrt(n) of the diagonal for good distribution
        let cell_size = (diagonal / (points.len() as f32).sqrt()) * 0.5;
        
        let mut grid = HashMap::with_capacity_and_hasher(points.len() * points.len(), Default::default());
        
        for &point in points {
            let cell = Self::point_to_cell(point, cell_size, min_x, min_y);
            grid.entry(cell).or_insert_with(Vec::new).push(point);
        }
        
        SpatialGrid { 
            grid, 
            cell_size,
            min_x,
            min_y,
        }
    }
    
    fn point_to_cell(point: Point, cell_size: f32, min_x: f32, min_y: f32) -> (i32, i32) {
        (
            ((point.x - min_x) / cell_size) as i32,
            ((point.y - min_y) / cell_size) as i32
        )
    }
    
    pub fn query_edge_candidates(&self, edge_start: Point, edge_end: Point, max_distance: f32) -> Vec<Point> {
        let mut candidates = Vec::new();
        
        // Sample multiple points along the edge for better candidate finding

        /*old
        let num_samples = 5;
        for i in 0..=num_samples {
            let t = i as f32 / num_samples as f32;
            let sample_point = Point {
                x: edge_start.x + t * (edge_end.x - edge_start.x),
                y: edge_start.y + t * (edge_end.y - edge_start.y),
            };
            
            let nearby = self.query_radius(sample_point, max_distance);
            for point in nearby {
                // Avoid duplicates by checking if already in candidates
                if !candidates.contains(&point) {
                    candidates.push(point);
                }
            }
        }
        */
        let points_to_test = [edge_start, edge_end, Point {x: (edge_end.x + edge_start.x) / 2.0, y: (edge_end.y + edge_start.y) / 2.0}];
        for point in points_to_test.iter(){
            let nearby = self.query_radius(point.clone(), max_distance);
            for candidate in nearby{
                if !candidates.contains(&candidate){
                    candidates.push(candidate);
                }
            }
        }
        return candidates
    }
    
    pub fn query_radius(&self, center: Point, radius: f32) -> Vec<Point> {
        let cells_to_check = ((radius / self.cell_size).ceil() as i32).max(1);
        let center_cell = Self::point_to_cell(center, self.cell_size, self.min_x, self.min_y);
        
        let mut candidates = Vec::new();
        let radius_sq = radius * radius;
        
        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(points) = self.grid.get(&cell) {
                    for &point in points {
                        let dist_sq = (point.x - center.x).powi(2) + (point.y - center.y).powi(2);
                        if dist_sq <= radius_sq {
                            candidates.push(point);
                        }
                    }
                }
            }
        }
        candidates
    }
    
    pub fn remove_point(&mut self, point: Point) {
        let cell = Self::point_to_cell(point, self.cell_size, self.min_x, self.min_y);
        if let Some(points) = self.grid.get_mut(&cell) {
            points.retain(|&p| p != point);
            // Remove empty cells to keep the grid clean
            if points.is_empty() {
                self.grid.remove(&cell);
            }
        }
    }
    
    pub fn contains_point(&self, point: Point) -> bool {
        let cell = Self::point_to_cell(point, self.cell_size, self.min_x, self.min_y);
        if let Some(points) = self.grid.get(&cell) {
            points.contains(&point)
        } else {
            return false
        }
    }
}

// Calculate reasonable search radius based on hull size
pub fn calculate_search_radius(hull: &[Point]) -> f32 {
    if hull.len() < 3 {
        return 100.0; // Default fallback
    }
    
    // Calculate average edge length in current hull
    let mut total_length = 0.0;
    for i in 0..hull.len() {
        let current = hull[i];
        let next = hull[(i + 1) % hull.len()];
        let dx = next.x - current.x;
        let dy = next.y - current.y;
        total_length += (dx * dx + dy * dy).sqrt();
    }
    
    let avg_edge_length = total_length / hull.len() as f32;
    
    // Search radius should be proportional to average edge length
    // This prevents searching too far from relevant edges
    return avg_edge_length * 2.0
}