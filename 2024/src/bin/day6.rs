use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::read_to_string,
};

use clap::Parser;
use ndarray::RawViewRepr;
use nom::{
    IResult,
    branch::alt,
    character::complete::{self, newline},
    combinator::{map, opt, value},
    multi::many1,
    sequence::terminated,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy)]
enum RawTile {
    Empty,
    Occupied,
    GuardUp,
    GuardDown,
    GuardLeft,
    GuardRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Empty,
    Occupied,
}

impl From<RawTile> for Tile {
    fn from(tile: RawTile) -> Self {
        match tile {
            RawTile::Empty => Tile::Empty,
            RawTile::Occupied => Tile::Occupied,
            RawTile::GuardUp => Tile::Empty,
            RawTile::GuardDown => Tile::Empty,
            RawTile::GuardLeft => Tile::Empty,
            RawTile::GuardRight => Tile::Empty,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Guard {
    pos: (i32, i32),
    direction: Direction,
}

impl Guard {
    fn peek(&self) -> (i32, i32) {
        let Guard {
            pos: (x, y),
            direction,
        } = self;
        let (x, y) = (*x, *y);
        match direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        }
    }

    fn rotate(self, tile_ahead: Tile) -> Option<Self> {
        let Guard {
            pos: (x, y),
            direction,
        } = self;
        let direction = match (tile_ahead, direction) {
            (Tile::Occupied, Direction::Up) => Direction::Right,
            (Tile::Occupied, Direction::Right) => Direction::Down,
            (Tile::Occupied, Direction::Down) => Direction::Left,
            (Tile::Occupied, Direction::Left) => Direction::Up,
            (Tile::Empty, dir) => return None,
        };
        Some(Self {
            pos: (x, y),
            direction,
        })
    }

    fn march(self) -> Self {
        Guard {
            pos: self.peek(),
            direction: self.direction,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct Grid<T> {
    letter: Vec<Vec<T>>,
    height: i32,
    width: i32,
}

impl<T: Copy> Grid<T> {
    fn new(letter: Vec<Vec<T>>) -> Self {
        let height = letter.len() as i32;
        let width = letter[0].len() as i32;
        Self {
            letter,
            width,
            height,
        }
    }
    fn get(&self, x: i32, y: i32) -> Option<T> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some(self.letter[y as usize][x as usize])
    }
    fn set(&mut self, x: i32, y: i32, s: T) -> Option<()> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        self.letter[y as usize][x as usize] = s;
        Some(())
    }
}

fn grid(input: &str) -> IResult<&str, Grid<RawTile>> {
    map(
        many1(terminated(
            many1(alt((
                value(RawTile::Empty, complete::char('.')),
                value(RawTile::Occupied, complete::char('#')),
                value(RawTile::GuardUp, complete::char('^')),
                value(RawTile::GuardDown, complete::char('v')),
                value(RawTile::GuardLeft, complete::char('<')),
                value(RawTile::GuardRight, complete::char('>')),
            ))),
            opt(newline),
        )),
        Grid::new,
    )(input)
}

fn parse_guard(grid: Grid<RawTile>) -> (Grid<Tile>, Option<Guard>) {
    let guard = grid
        .letter
        .iter()
        .enumerate()
        .find_map(|(j, row)| {
            row.iter().enumerate().find_map(|(i, x)| match x {
                RawTile::GuardUp => Some((i, j, Direction::Up)),
                RawTile::GuardDown => Some((i, j, Direction::Down)),
                RawTile::GuardLeft => Some((i, j, Direction::Left)),
                RawTile::GuardRight => Some((i, j, Direction::Right)),
                _ => None,
            })
        })
        .map(|(x, y, d)| Guard {
            pos: (x as i32, y as i32),
            direction: d,
        });
    let new_grid = Grid::new(
        grid.letter
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|x| match x {
                        RawTile::Empty => Tile::Empty,
                        RawTile::Occupied => Tile::Occupied,
                        RawTile::GuardDown => Tile::Empty,
                        RawTile::GuardUp => Tile::Empty,
                        RawTile::GuardLeft => Tile::Empty,
                        RawTile::GuardRight => Tile::Empty,
                    })
                    .collect()
            })
            .collect(),
    );
    (new_grid, guard)
}

fn solve_part1(grid: &Grid<Tile>, mut guard: Guard) -> i32 {
    let mut visited = Vec::new();
    let (mut x, mut y) = guard.peek();
    visited.push(guard.pos);
    while let Some(tile_ahead) = grid.get(x, y) {
        if let Some(g) = guard.rotate(tile_ahead) {
            guard = g;
            continue;
        };
        guard = guard.march();
        visited.push(guard.pos);
        (x, y) = guard.peek();
    }
    visited.into_iter().collect::<BTreeSet<_>>().len() as i32
}

fn print_loop(grid: &Grid<Tile>, prevs: &[Guard]) {
    let mut h = HashMap::new();
    for Guard { pos, direction } in prevs {
        h.insert(pos, direction);
    }
    for y in 0..grid.height {
        for x in 0..grid.width {
            print!(
                "{}",
                match grid.get(x, y) {
                    Some(Tile::Occupied) => "#",
                    Some(Tile::Empty) => {
                        match h.get(&(x, y)) {
                            Some(Direction::Down) => "v",
                            Some(Direction::Up) => "^",
                            Some(Direction::Left) => "<",
                            Some(Direction::Right) => ">",
                            None => ".",
                        }
                    }
                    None => {
                        panic!("moo")
                    }
                }
            );
        }
        print!("\n");
    }
}

fn has_loop(
    grid: &Grid<Tile>,
    mut guard: Guard,
    iterations: i32,
    orig_x: i32,
    orig_y: i32,
) -> bool {
    let mut visited = HashSet::new();
    let mut prev = Vec::new();
    let (mut x, mut y) = guard.peek();
    while let Some(tile_ahead) = grid.get(x, y) {
        if let Some(g) = guard.rotate(tile_ahead) {
            guard = g;
            continue;
        };
        guard = guard.march();
        if visited.contains(&guard) {
            print_loop(grid, &prev);
            return true;
        }
        visited.insert(guard);
        prev.push(guard);
        (x, y) = guard.peek();
    }
    false
}

fn solve_part2(grid: &Grid<Tile>, guard: Guard) -> i32 {
    let mut loop_count = 0;
    let mut iterations = 0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            iterations += 1;
            if iterations != 13808 {
                continue;
            }
            if guard.pos == (x, y) {
                continue;
            }
            if grid.get(x, y) == Some(Tile::Occupied) {
                continue;
            }
            let mut new_grid = grid.clone();
            new_grid.set(x, y, Tile::Occupied).unwrap();
            if has_loop(&new_grid, guard, iterations, x, y) {
                loop_count += 1;
            }
        }
    }
    loop_count
}

fn main() {
    let args = Cli::parse();
    let content = read_to_string(args.path).expect("could not read file");
    let (remain, grid) = grid(&content).unwrap();
    let (grid, guard) = parse_guard(grid);
    let guard = guard.expect("No guard found");
    println!("{}", solve_part1(&grid, guard));
    println!("{}", solve_part2(&grid, guard));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop() {}
}
