use std::time::Instant;
use indicatif::ProgressBar;
use rayon::prelude::*;
mod reader;
mod shared;


#[derive(Debug)]
struct InsertPointResult {
    lda: f32,
    best_a: shared::Point,
    best_c: shared::Point,
}

fn calc_dist(a: shared::Point, b: shared::Point) -> f32 {
    return ((a.x - b.x)*(a.x - b.x) + (a.y - b.y)*(a.y - b.y)).sqrt();
}


fn convex_hull(points: &[shared::Point]) -> Vec<shared::Point> {
    let mut points = points.to_vec();


    if points.len() <= 1 {
        return points;
    }


    points.sort_by(|a, b| {
        return a.x.partial_cmp(&b.x).unwrap()
            .then(a.y.partial_cmp(&b.y).unwrap());
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


    // Remove last point from both to avoid duplication
    lower.pop();
    upper.pop();


    lower.extend(upper);
    return lower;
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

    return (diff_x * diff_x + diff_y * diff_y).sqrt()
}





//Much faster approxomation
#[inline(always)]
fn fast_acos(x: f32) -> f32 {
    let x_abs = f32::from_bits(x.to_bits() & 0x7FFF_FFFF);
    let sqrt_term = (1.0 - x_abs).sqrt();

    // Use fma to improve both performance and accuracy
    let base = x_abs.mul_add(
        x_abs.mul_add(
            x_abs.mul_add(-0.0187293, 0.0742610),
            -0.2121144,
        ),
        1.5707288,
    );

    let ret = base * sqrt_term;

    if x.is_sign_negative() {
        std::f32::consts::PI - ret
    } else {
        ret
    }
}


fn lda(a: &shared::Point, b: &shared::Point, c: &shared::Point) -> f32 {
    let ab = calc_dist(*a, *b);
    let bc = calc_dist(*b, *c);
    let ac = calc_dist(*a, *c);


    let numerator = (ac * ac) + (bc * bc) - (ab * ab);
    let denominator = 2.0 * bc * ac;
    let cosine = numerator / denominator;
    let acos = fast_acos(cosine);
    let dist = point_line(a, b, c);
    return acos / dist;
}


fn insert_point(hull: &[shared::Point], inner_points: &[shared::Point]) -> InsertPointResult {    
    inner_points.par_iter().map(|&c| {            
        let mut best_lda = -1.0;            
        let mut best_a = shared::Point { x: 0.0, y: 0.0 };              
        for j in 0..(hull.len() - 1) {                
            let a = hull[j];                
            let b = hull[j + 1];                
            let curr_lda = lda(&a, &b, &c);                  
            if curr_lda > best_lda {                    
                best_lda = curr_lda;                    
                best_a = a;                
            }            
        }// wraparound edge            
        let final_lda = lda(&hull[hull.len() - 1], &hull[0], &c);            
        if final_lda > best_lda {                
            best_lda = final_lda;                
            best_a = hull[hull.len() - 1];            
        }              
        InsertPointResult {                
            lda: best_lda,                
            best_a: best_a,                
            best_c: c,            
        }        
    })        
    .reduce(            
        || InsertPointResult {                
            lda: -1.0,                
            best_a: shared::Point { x: 0.0, y: 0.0 },                
            best_c: shared::Point { x: 0.0, y: 0.0 },            
        },|a, b|
         if a.lda > b.lda { a } else { b },        
    )}


fn update_hull(
    result: &InsertPointResult,
    hull: &mut Vec<shared::Point>,
    inner_hull: &mut Vec<shared::Point>,
) {
    // Remove best_c from inner_hull
    if let Some(pos) = inner_hull.iter().position(|&p| p == result.best_c) {
        inner_hull.remove(pos);
    }


    // Find the position of best_a in hull
    if let Some(pos) = hull.iter().position(|&p| p == result.best_a) {
        // Insert best_c after best_a
        hull.insert(pos + 1, result.best_c);
    }
}
fn path_dist(path: &[shared::Point]) -> f32{
    let mut sum = 0.0;
    for i in 0 .. &path.len() - 1{
        sum += calc_dist(path[i], path[i+1]);
    }
    sum += calc_dist(path[path.len() - 1], path[0]);
    return sum;
}
fn main() {
    //When I remove num_threads it does
    rayon::ThreadPoolBuilder::new().num_threads(20).build_global().unwrap();
   
    let points: Vec<shared::Point> = reader::parse_file(&reader::read_file());


    let start = Instant::now();


    let mut hull = convex_hull(&points);
    let mut inner_hull = reader::vec_diff(&points, &hull);
    let pb = ProgressBar::new(inner_hull.len().try_into().unwrap());
    while inner_hull.len() > 0{
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



