use std::{env::current_dir, fs::read_to_string, path::PathBuf};

use ahash::{HashSet, HashSetExt};
use clap::Parser;
use itertools::Itertools;
use nom::{
    IResult,
    branch::alt,
    character::complete::{self, newline},
    combinator::{map, opt},
    multi::many1,
    sequence::terminated,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug)]
struct Grid {
    letter: Vec<Vec<char>>,
    height: i32,
    width: i32,
}

impl Grid {
    fn new(letter: Vec<Vec<char>>) -> Self {
        let height = letter.len() as i32;
        let width = letter[0].len() as i32;
        Self {
            letter,
            width,
            height,
        }
    }
    fn get(&self, x: i32, y: i32) -> Option<char> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some(self.letter[y as usize][x as usize])
    }
}

// parse a grid of characters
fn grid(input: &str) -> IResult<&str, Grid> {
    map(
        many1(terminated(
            many1(alt((
                complete::char('X'),
                complete::char('M'),
                complete::char('A'),
                complete::char('S'),
            ))),
            opt(newline),
        )),
        Grid::new,
    )(input)
}

fn directions(word: &[char]) -> Vec<Vec<(i32, i32, char)>> {
    let mut result = Vec::new();
    for i in -1..=1 {
        for j in -1..=1 {
            let mut v = Vec::new();
            for (direction, c) in word.iter().enumerate() {
                v.push((direction as i32 * i, direction as i32 * j, *c));
            }
            result.push(v);
        }
    }
    result
}

fn mas_directions() -> Vec<Vec<(i32, i32, char)>> {
    let mut result = Vec::new();
    let mut v = Vec::new();
    for (i, c) in (-1..=1).zip(['M', 'A', 'S']) {
        v.push((i, i, c));
        v.push((i, -i, c));
    }
    result.push(v);

    let mut v = Vec::new();
    for (i, c) in (-1..=1).zip(['M', 'A', 'S']) {
        v.push((i, i, c));
        v.push((-i, i, c));
    }
    result.push(v);

    let mut v = Vec::new();
    for (i, c) in (-1..=1).zip(['M', 'A', 'S']) {
        v.push((-i, -i, c));
        v.push((i, -i, c));
    }
    result.push(v);

    let mut v = Vec::new();
    for (i, c) in (-1..=1).zip(['M', 'A', 'S']) {
        v.push((-i, -i, c));
        v.push((-i, i, c));
    }
    result.push(v);
    result
}

fn solve(grid: &Grid, pattern: &[Vec<(i32, i32, char)>]) -> i32 {
    let mut count = 0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            for path in pattern.into_iter() {
                count += if path.iter().all(|(i, j, c)| match grid.get(x + i, y + j) {
                    Some(gc) if gc == *c => true,
                    Some(gc) => false,
                    None => false,
                }) {
                    1
                } else {
                    0
                };
            }
        }
    }
    count
}
fn main() {
    let args = Cli::parse();
    let content = read_to_string(args.path).expect("could not read file");
    let (_, grid) = grid(&content).expect("parse error");
    let part1 = solve(&grid, &directions(&['X', 'M', 'A', 'S']));
    println!("{part1:?}");
    let part2 = solve(&grid, &mas_directions());
    println!("{part2:?}");
}
