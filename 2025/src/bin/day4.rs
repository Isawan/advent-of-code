use advent_of_code::grid::Grid;
use ahash::{HashMap, HashMapExt};
use itertools::Itertools;
use rayon::iter::Empty;
use std::cmp::max;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Tile {
    Roll,
    Empty,
}

fn parse(input: &str) -> Grid<Tile> {
    Grid::new(
        input
            .lines()
            .map(|line| {
                line.chars()
                    .map(|x| match x {
                        '@' => Tile::Roll,
                        '.' => Tile::Empty,
                        _ => panic!("invalid"),
                    })
                    .collect()
            })
            .collect(),
    )
}

fn part1(input: &str) -> i32 {
    let grid = parse(input);
    let mut accessible_count = 0;
    for (x, y, tile) in grid.all() {
        if tile == Tile::Empty {
            continue;
        }
        let mut count = 0;
        for (i, j) in (-1..=1).cartesian_product(-1..=1) {
            if i == 0 && j == 0 {
                continue;
            }
            count += if grid.get(x + i, y + j).unwrap_or(Tile::Empty) == Tile::Roll {
                1
            } else {
                0
            }
        }
        accessible_count += if count < 4 {
            println!("{x} {y}");
            1
        } else {
            0
        }
    }
    accessible_count
}

fn main() {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    println!("Part 1: {}", part1(&input));
    // println!("Part 2: {}", solve(12, &input));
}

#[cfg(test)]
mod tests {}
