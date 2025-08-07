use std::env;
use std::fs;
use crate::shared;

pub fn read_file() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }
    if &args[1] == "help" || &args[1] == "--help" || &args[1] == "-help"{
        println!("To use run ./tsp.exe <PATH TO .tsp FILE>");
        std::process::exit(1);
    }
    let filename = &args[1];
    return fs::read_to_string(filename).unwrap();
}

pub fn parse_num(input: &String) -> f32 {
    // Rust handles scientific notation parsing directly, so:
    return input.parse::<f32>().unwrap();
}
pub fn should_log() ->bool {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3{
        return true;
    }
    if args[2] == "--no-log"{
        return false;
    }
    return true;
}
pub fn parse_file(file: &String) -> Vec<shared::Point> {
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
            to_return.push(shared::Point {
                x: parse_num(&split[1].to_string()),
                y: parse_num(&split[2].to_string()),
            });
        }
    }

    return to_return;
}

pub fn vec_diff(a: &[shared::Point], b: &[shared::Point]) -> Vec<shared::Point> {
    return a.iter()
        .filter(|item| !b.contains(item))
        .cloned()
        .collect();
}