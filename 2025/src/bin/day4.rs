use advent_of_code::grid::Grid;
use ahash::{HashMap, HashMapExt};
use itertools::Itertools;
use rayon::iter::Empty;
use std::{cmp::max, thread::AccessError};

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

fn get_accessible(grid: &Grid<Tile>) -> Vec<(i32, i32)> {
    let mut accessibles = Vec::new();
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
        if count < 4 {
            accessibles.push((x, y))
        }
    }
    accessibles
}

fn part1(input: &str) -> usize {
    let grid = parse(input);
    get_accessible(&grid).len()
}

fn part2(input: &str) -> usize {
    let mut grid = parse(input);
    let mut removed = 0;
    loop {
        let accessible = get_accessible(&grid);
        for (x, y) in accessible.iter() {
            grid.set(*x, *y, Tile::Empty);
        }
        if accessible.is_empty() {
            break;
        }
        removed += accessible.len();
    }
    removed
}

fn main() {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    println!("Part 1: {}", part1(&input));
    println!("Part 2: {}", part2(&input));
}

#[cfg(test)]
mod tests {}
