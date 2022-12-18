use lazy_static::lazy_static;
use nom::bytes::streaming::is_a;
use regex::{Regex, RegexBuilder};
use std::cmp::{max, min, Reverse};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::collections::{BTreeSet, BinaryHeap};
use std::iter;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
struct Commands {
    buffer: Vec<Direction>,
    index: usize,
    cycle: usize,
}

impl Iterator for Commands {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        let direction = self.buffer[self.index];
        self.index = (self.index + 1) % self.buffer.len();
        Some(direction)
    }
}

fn find_cycles(vector: &[impl Eq]) -> Vec<usize> {
    let mut cycles = Vec::new();
    cycles.push(vector.len());
    for shift in 1..vector.len() {
        let mut same = true;
        for index in 0..vector.len() {
            same = same && (vector[index] == vector[(index + shift) % vector.len()]);
        }
        if same {
            cycles.push(shift);
        }
    }
    cycles.sort();
    cycles
}

fn parse(input: &str) -> Commands {
    let buffer: Vec<Direction> = input
        .chars()
        .filter_map(|c| match c {
            '<' => Some(Direction::Left),
            '>' => Some(Direction::Right),
            _ => None,
        })
        .collect();
    let cycle = buffer.len();
    Commands {
        buffer,
        cycle,
        index: 0,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Shape {
    rock_positions: &'static [(usize, usize)],
}

impl Shape {
    fn positions(&self) -> impl Iterator<Item = &(usize, usize)> {
        self.rock_positions.iter()
    }

    fn vertical_line() -> Self {
        Shape {
            rock_positions: &[(0, 0), (0, 1), (0, 2), (0, 3)],
        }
    }
    fn cross() -> Self {
        Shape {
            rock_positions: &[(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
        }
    }
    fn ell() -> Self {
        Shape {
            rock_positions: &[(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)],
        }
    } // CAREFUL NEED TO HANDLE ASYMMETRY. Ignore for now, revisit when the world is implemented.
    fn horizontal_line() -> Self {
        Shape {
            rock_positions: &[(0, 0), (1, 0), (2, 0), (3, 0)],
        }
    }
    fn square() -> Self {
        Shape {
            rock_positions: &[(0, 0), (0, 1), (1, 0), (1, 1)],
        }
    }
}

#[derive(Debug)]
struct ShapeGenerator {
    index: usize,
}

impl ShapeGenerator {
    fn new() -> Self {
        ShapeGenerator { index: 4 }
    }
}

impl Iterator for ShapeGenerator {
    type Item = Shape;
    fn next<'a>(&'a mut self) -> Option<Self::Item> {
        self.index = (self.index + 1) % 5;
        let shape = match self.index {
            0 => Shape::horizontal_line(),
            1 => Shape::cross(),
            2 => Shape::ell(),
            3 => Shape::vertical_line(),
            4 => Shape::square(),
            _ => panic!("out of bounds"),
        };
        Some(shape)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Position(usize, usize);

impl Position {
    fn x(&self) -> usize {
        self.0
    }
    fn y(&self) -> usize {
        self.1
    }
}

#[derive(Debug)]
struct Chamber {
    placements: Vec<bool>,
    width: usize,
}

impl Chamber {
    fn new() -> Self {
        Self {
            placements: Vec::new(),
            width: 7,
        }
    }
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.placements.len() / self.width
    }
    fn get(&self, pos: &Position) -> bool {
        let index = (pos.y() * self.width()) + pos.x();
        self.placements.get(index).copied().unwrap_or(false)
    }
    fn set(mut self, shape: &Shape, pos: Position) -> Self {
        let set_pos = shape.positions().map(|(x, y)| (x + pos.x(), y + pos.y()));
        for each_pos in set_pos {
            // allocate layer as needed to accommodate for higher positions
            while each_pos.1 >= self.height() {
                self.placements.extend(iter::repeat(false).take(self.width));
            }
            assert_eq!(
                self.placements[(each_pos.1 * self.width()) + each_pos.0],
                false
            );
            let index = (each_pos.1 * self.width()) + each_pos.0;
            self.placements[index] = true;
        }
        self
    }
}

impl Position {
    fn new(chamber: &Chamber, pos: (usize, usize)) -> Option<Self> {
        if pos.0 >= chamber.width() {
            return None;
        }
        Some(Self(pos.0, pos.1))
    }
    fn new_shape_position(chamber: &Chamber) -> Self {
        Position::new(chamber, (2, chamber.height() + 3)).unwrap()
    }
}

fn try_position(chamber: &Chamber, shape: &Shape, pos: Position) -> Option<Position> {
    shape
        .positions()
        .map(|(x, y)| Position::new(chamber, (x + pos.x(), y + pos.y())))
        .map(|p| p.filter(|p| !chamber.get(p)))
        .all(|p| p.is_some())
        .then_some(pos)
}

fn is_valid_position(chamber: &Chamber, shape: &Shape, pos: &Position) -> bool {
    shape
        .positions()
        .map(|(x, y)| Position::new(chamber, (x + pos.x(), y + pos.y())))
        .map(|p| p.filter(|p| !chamber.get(p)))
        .all(|p| p.is_some())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RestState {
    Continue(Position),
    Rest(Position),
}

fn move_horizontal(chamber: &Chamber, pos: Position, shape: &Shape, dir: Direction) -> Position {
    match dir {
        Direction::Left => pos.x().checked_sub(1),
        Direction::Right => pos.x().checked_add(1),
    }
    .and_then(|x| Position::new(&chamber, (x, pos.y())))
    .filter(|p| is_valid_position(&chamber, &shape, p))
    .unwrap_or(pos)
}

fn move_vertical(chamber: &Chamber, pos: Position, shape: &Shape) -> Option<Position> {
    pos.y()
        .checked_sub(1)
        .and_then(|y| Position::new(chamber, (pos.x(), y)))
        .filter(|pos| is_valid_position(&chamber, &shape, pos))
}

fn run_round(
    chamber: Chamber,
    shape: &Shape,
    dir: &mut impl Iterator<Item = Direction>,
) -> Chamber {
    let mut pos = Position::new_shape_position(&chamber);
    let end_pos = loop {
        let next_pos = move_horizontal(&chamber, pos, shape, dir.next().unwrap());
        pos = match move_vertical(&chamber, next_pos.clone(), shape) {
            Some(p) => p,
            None => break next_pos,
        };
    };
    chamber.set(shape, end_pos)
}

type Height = usize;
fn run(input: &str, times: u32) -> Height {
    let mut chamber = Chamber::new();
    let mut directions = parse(input);
    let mut shape_generator = ShapeGenerator::new();

    for _ in 0..times {
        chamber = run_round(chamber, &shape_generator.next().unwrap(), &mut directions)
    }
    chamber.height()
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    println!("solution1 {:?}", run(&input, 2022));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use regex::CaptureMatches;

    use super::*;

    #[test]
    fn test_set_line_horizontal() {
        let chamber = Chamber::new();
        let place_position = Position::new(&chamber, (1, 0)).unwrap();
        let chamber = chamber.set(&Shape::horizontal_line(), place_position);
        let get = |p| chamber.get(&Position::new(&chamber, p).unwrap());
        assert_eq!(get((0, 0)), false);
        assert_eq!(get((1, 0)), true);
        assert_eq!(get((2, 0)), true);
        assert_eq!(get((3, 0)), true);
        assert_eq!(get((4, 0)), true);
        assert_eq!(get((5, 0)), false);
        assert_eq!(get((6, 0)), false);
        assert_eq!(get((0, 1)), false);
        assert_eq!(get((1, 1)), false);
        assert_eq!(get((2, 1)), false);
        assert_eq!(get((3, 1)), false);
        assert_eq!(get((4, 1)), false);
        assert_eq!(get((5, 1)), false);
        assert_eq!(get((6, 1)), false);
    }

    #[test]
    fn test_set_line_vertical() {
        let chamber = Chamber::new();
        let place_position = Position::new(&chamber, (1, 0)).unwrap();
        let chamber = chamber.set(&Shape::vertical_line(), place_position);
        let get = |p| chamber.get(&Position::new(&chamber, p).unwrap());
        assert_eq!(get((0, 0)), false);
        assert_eq!(get((0, 1)), false);
        assert_eq!(get((0, 2)), false);
        assert_eq!(get((0, 3)), false);
        assert_eq!(get((0, 4)), false);
        assert_eq!(get((1, 0)), true);
        assert_eq!(get((1, 1)), true);
        assert_eq!(get((1, 2)), true);
        assert_eq!(get((1, 3)), true);
        assert_eq!(get((1, 4)), false);
        assert_eq!(get((2, 0)), false);
        assert_eq!(get((2, 1)), false);
        assert_eq!(get((2, 2)), false);
        assert_eq!(get((2, 3)), false);
        assert_eq!(get((2, 4)), false);
    }

    #[test]
    fn test_set_line_ell() {
        let chamber = Chamber::new();
        let place_position = Position::new(&chamber, (1, 0)).unwrap();
        let chamber = chamber.set(&Shape::ell(), place_position);
        let get = |p| chamber.get(&Position::new(&chamber, p).unwrap());
        assert_eq!(get((0, 0)), false);
        assert_eq!(get((0, 1)), false);
        assert_eq!(get((0, 2)), false);
        assert_eq!(get((0, 3)), false);
        assert_eq!(get((1, 0)), true);
        assert_eq!(get((1, 1)), false);
        assert_eq!(get((1, 2)), false);
        assert_eq!(get((1, 3)), false);
        assert_eq!(get((2, 0)), true);
        assert_eq!(get((2, 1)), false);
        assert_eq!(get((2, 2)), false);
        assert_eq!(get((2, 3)), false);
        assert_eq!(get((3, 0)), true);
        assert_eq!(get((3, 1)), true);
        assert_eq!(get((3, 2)), true);
        assert_eq!(get((3, 3)), false);
        assert_eq!(get((4, 0)), false);
        assert_eq!(get((4, 1)), false);
        assert_eq!(get((4, 2)), false);
        assert_eq!(get((4, 3)), false);
    }

    #[test]
    fn test_round() {
        let chamber = Chamber::new();
        let mut directions = parse(include_str!("../../input/day17-test"));
        let chamber = run_round(chamber, &Shape::horizontal_line(), &mut directions);
        let get = |p| chamber.get(&Position::new(&chamber, p).unwrap());
        assert_eq!(get((0, 0)), false);
        assert_eq!(get((1, 0)), false);
        assert_eq!(get((2, 0)), true);
        assert_eq!(get((3, 0)), true);
        assert_eq!(get((4, 0)), true);
        assert_eq!(get((5, 0)), true);
        assert_eq!(get((6, 0)), false);
        assert_eq!(get((0, 1)), false);
        assert_eq!(get((1, 1)), false);
        assert_eq!(get((2, 1)), false);
        assert_eq!(get((3, 1)), false);
        assert_eq!(get((4, 1)), false);
        assert_eq!(get((5, 1)), false);
        assert_eq!(get((6, 1)), false);
    }

    #[test]
    fn test_valid_position() {
        let chamber = Chamber::new();
        let shape = &Shape::horizontal_line();
        assert_eq!(
            is_valid_position(&chamber, shape, &Position::new(&chamber, (3, 2)).unwrap()),
            true
        );
        assert_eq!(
            is_valid_position(&chamber, shape, &Position::new(&chamber, (4, 2)).unwrap()),
            false
        );
    }

    #[test]
    fn test_direction_generator() {
        let mut directions = parse(include_str!("../../input/day17-test"));
        assert_eq!(directions.next(), Some(Direction::Right));
        assert_eq!(directions.next(), Some(Direction::Right));
        assert_eq!(directions.next(), Some(Direction::Right));
        assert_eq!(directions.next(), Some(Direction::Left));
    }

    #[test]
    fn test_shape_generator() {
        let mut shapes = ShapeGenerator::new();
        assert_eq!(shapes.next(), Some(Shape::horizontal_line()));
        assert_eq!(shapes.next(), Some(Shape::cross()));
        assert_eq!(shapes.next(), Some(Shape::ell()));
        assert_eq!(shapes.next(), Some(Shape::vertical_line()));
        assert_eq!(shapes.next(), Some(Shape::square()));
        assert_eq!(shapes.next(), Some(Shape::horizontal_line()));
    }

    #[test]
    fn test_example() {
        assert_eq!(run(include_str!("../../input/day17-test"), 2022), 3068);
    }

    #[test]
    fn test_find_cycles() {
        assert_eq!(find_cycles(&[1, 0, 0, 1, 0, 0]), vec![3, 6]);
    }
}
