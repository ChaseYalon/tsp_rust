use std::env;
use std::fs;
use std::time::Instant;
use indicatif::ProgressBar;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct insert_point_result {
    lda: f32,
    best_a: Point,
    best_c: Point,
}

impl Point {
    fn cross(o: &Point, a: &Point, b: &Point) -> f32 {
        return (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    }
}

fn calc_dist(a: Point, b: Point) -> f32 {
    return ((a.x - b.x)*(a.x - b.x) + (a.y - b.y)*(a.y - b.y)).sqrt();
}

fn convex_hull(points: &[Point]) -> Vec<Point> {
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
            && Point::cross(&lower[lower.len() - 2], &lower[lower.len() - 1], p) <= 0.0
        {
            lower.pop();
        }
        lower.push(*p);
    }

    let mut upper = Vec::new();
    for p in points.iter().rev() {
        while upper.len() >= 2
            && Point::cross(&upper[upper.len() - 2], &upper[upper.len() - 1], p) <= 0.0
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

fn read_file() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    return fs::read_to_string(filename).unwrap();
}

fn parse_num(input: &String) -> f32 {
    // Rust handles scientific notation parsing directly, so:
    return input.parse::<f32>().unwrap();
}

fn parse_file(file: &String) -> Vec<Point> {
    let parts: Vec<&str> = file.split("NODE_COORD_SECTION").collect();
    if parts.len() < 2 {
        return vec![];
    }
    let file_contents_2 = parts[1];
    let file_contents_3 = file_contents_2.split("EOF").next().unwrap_or("");

    let lines: Vec<&str> = file_contents_3.lines().collect();
    let mut to_return = Vec::new();

    for line in lines {
        let split: Vec<&str> = line.split_whitespace().collect();
        if split.len() >= 3 {
            to_return.push(Point {
                x: parse_num(&split[1].to_string()),
                y: parse_num(&split[2].to_string()),
            });
        }
    }

    return to_return;
}

fn vec_diff(a: &[Point], b: &[Point]) -> Vec<Point> {
    return a.iter()
        .filter(|item| !b.contains(item))
        .cloned()
        .collect();
}
fn point_line(a: Point, b: Point, c: Point) -> f32 {
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
    let x_abs = x.abs();
    let sqrt_term = (1.0 - x_abs).sqrt();
    let base = ((-0.0187293 * x_abs + 0.0742610) * x_abs - 0.2121144) * x_abs + 1.5707288;
    let ret = base * sqrt_term;

    if x < 0.0 {
        return std::f32::consts::PI - ret;
    } else {
        return ret;
    }
}


fn lda(a: &Point, b: &Point, c: &Point) -> f32 {
    let ab = calc_dist(*a, *b);
    let bc = calc_dist(*b, *c);
    let ac = calc_dist(*a, *c);

    let numerator = (ac * ac) + (bc * bc) - (ab * ab);
    let denominator = 2.0 * bc * ac;
    let cosine = numerator / denominator;
    let acos = fast_acos(cosine);
    let dist = point_line(*a, *b, *c);
    return acos / dist;
}

fn insert_point(hull: &[Point], inner_points: &[Point]) -> insert_point_result {     
    inner_points.par_iter().map(|&c| {             
        let mut best_lda = -1.0;             
        let mut best_a = Point { x: 0.0, y: 0.0 };              
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
        insert_point_result {                 
            lda: best_lda,                 
            best_a: best_a,                 
            best_c: c,             
        }         
    })         
    .reduce(             
        || insert_point_result {                 
            lda: -1.0,                 
            best_a: Point { x: 0.0, y: 0.0 },                 
            best_c: Point { x: 0.0, y: 0.0 },             
        },|a, b|
         if a.lda > b.lda { a } else { b },         
    )}

fn update_hull(
    result: &insert_point_result,
    hull: &mut Vec<Point>,
    inner_hull: &mut Vec<Point>,
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
fn path_dist(path: &[Point]) -> f32{
    let mut sum = 0.0;
    for i in 0 .. &path.len() - 1{
        sum += calc_dist(path[i], path[i+1]);
    }
    sum += calc_dist(path[path.len() - 1], path[0]);
    return sum;
}
fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(20).build_global().unwrap();
   
    let points: Vec<Point> = parse_file(&read_file());

    let start = Instant::now();

    let mut hull = convex_hull(&points);
    let mut inner_hull = vec_diff(&points, &hull);
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
