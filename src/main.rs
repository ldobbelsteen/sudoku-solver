use core::panic;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::{env, fs};
use sudoku_solver::Solver;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("No input file specified!")
    }
    if args.len() > 3 {
        panic!("Too many arguments specified!")
    }

    let input = args.get(1).unwrap();
    let output = args.get(2);

    let input_file = File::open(input).unwrap();
    let mut output_file = if let Some(output) = output {
        if Path::new(output).exists() {
            fs::remove_file(output).unwrap();
        }
        Some(File::create(output).unwrap())
    } else {
        None
    };

    let mut num_solved = 0;
    let mut num_solved_without_brute_force = 0;

    let reader = BufReader::new(input_file);
    for line in reader.lines() {
        let puzzle: Vec<char> = line.unwrap().chars().collect();
        let solution = Solver::solve(puzzle).unwrap();

        num_solved += 1;
        if !solution.used_brute_force {
            num_solved_without_brute_force += 1;
        }

        if let Some(output_file) = &mut output_file {
            let _ = output_file
                .write((solution.row_representation() + "\n").as_bytes())
                .unwrap();
        }
    }

    println!("Input file: {}", input);
    if let Some(output) = output {
        println!("Output file: {}", output);
    }
    println!("Total solved: {}", num_solved);
    println!("Without brute-force: {}", num_solved_without_brute_force);
}
