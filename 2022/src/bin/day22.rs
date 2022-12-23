use nom::{branch::alt, bytes::complete::tag, combinator::map, multi::many1, IResult};
use regex::internal::Inst;
use std::{cmp::min, time::Instant};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
enum Tile {
    Wall,
    Open,
    OffMap,
}

struct Grid {
    field: Vec<Tile>,
    width: i32,
    height: i32,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
struct Movement {
    orientation: (i32, i32),
    steps: i32,
}

enum Instruction {
    Step(i32),
    Right,
    Left,
}

fn instructions(ins: &str) -> IResult<&str, Vec<Instruction>> {
    many1(alt((
        map(tag("R"), |_| Instruction::Right),
        map(tag("L"), |_| Instruction::Left),
        map(nom::character::complete::i32, |i| Instruction::Step(i)),
    )))(ins)
}

fn parse(input: &str) -> (Grid, Vec<Instruction>) {
    let mut parts = input.split("\n\n");
    let ascii_grid = parts.next().unwrap();
    let text_instructions = parts.next().unwrap();

    (
        Grid::parse(ascii_grid),
        instructions(text_instructions).unwrap().1,
    )
}

fn instructions_to_movements(instructions: Vec<Instruction>) -> Vec<Movement> {
    let mut orient_index = 0isize;
    let orientations = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut moves = vec![];
    for ins in instructions.iter() {
        match ins {
            Instruction::Step(n) => moves.push(Movement {
                orientation: orientations[orient_index as usize],
                steps: *n,
            }),
            Instruction::Left => orient_index = (orient_index - 1).rem_euclid(4),
            Instruction::Right => orient_index = (orient_index + 1).rem_euclid(4),
        }
    }
    moves
}

impl Grid {
    fn parse(input: &str) -> Self {
        let height = input.lines().count() as i32;
        let width = input.find('\n').unwrap() as i32;
        let field = input
            .replace("\n", "")
            .as_str()
            .chars()
            .map(|c| match c {
                ' ' => Tile::OffMap,
                '.' => Tile::Open,
                '#' => Tile::Wall,
                _ => panic!("unexpected character"),
            }) // handle start and end position
            .collect();
        Grid {
            field,
            width,
            height,
        }
    }

    fn get(&self, pos: &(i32, i32)) -> Tile {
        if pos.0 >= self.width || pos.0 < 0 || pos.1 >= self.height || pos.1 < 0 {
            return Tile::OffMap;
        }
        self.field[((self.width * pos.1) + pos.0) as usize]
    }
    fn get_start_pos(&self) -> (i32, i32) {
        let y = 0;
        let x = (0..self.width).fold(i32::MAX, |a, i| match self.get(&(i, y)) {
            Tile::Open => min(a, i),
            _ => a,
        });
        (x, y)
    }
    fn step_position(&self, current: &(i32, i32), vector: &(i32, i32)) -> (i32, i32) {
        assert!(vector.0.abs() <= 1);
        assert!(vector.1.abs() <= 1);
        let mut new_position = (
            (current.0 + vector.0).rem_euclid(self.width),
            (current.1 + vector.1).rem_euclid(self.height),
        );
        loop {
            match self.get(&(new_position.0, new_position.1)) {
                Tile::OffMap => {
                    new_position = (
                        (new_position.0 + vector.0).rem_euclid(self.width),
                        (new_position.1 + vector.1).rem_euclid(self.height),
                    );
                }
                _ => {
                    return new_position;
                }
            }
        }
    }
}

fn move_once(grid: &Grid, current: &(i32, i32), vector: &(i32, i32)) -> (i32, i32) {
    let potential_pos = grid.step_position(&current, &vector);
    let next_pos = match grid.get(&potential_pos) {
        Tile::Open => potential_pos,
        Tile::Wall => current.clone(),
        Tile::OffMap => panic!("should not be able to get here"),
    };
    next_pos
}

fn perform_movement(grid: &Grid, current: &(i32, i32), movement: &Movement) -> (i32, i32) {
    let mut pos = current.clone();
    for _step in 0..movement.steps {
        pos = move_once(grid, &pos, &movement.orientation);
    }
    pos
}

fn score_orientation(orientation: &(i32, i32)) -> i32 {
    match orientation {
        (1, 0) => 0,
        (0, 1) => 1,
        (-1, 0) => 2,
        (0, -1) => 3,
        _ => panic!("unexpected"),
    }
}

fn calculate_password(input: &str) -> i32 {
    let (grid, instructions) = parse(input);
    let movements = instructions_to_movements(instructions);
    let mut pos = grid.get_start_pos();
    for mv in movements.iter() {
        pos = perform_movement(&grid, &pos, &mv);
    }
    (pos.1 + 1) * 1000 + (pos.0 + 1) * 4 + score_orientation(&movements.last().unwrap().orientation)
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    println!("solution 1: {}", calculate_password(&input));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solution() {
        let (grid, instructions) = parse(include_str!("../../input/day22-test"));
        let movements = instructions_to_movements(instructions);
        let mut pos = grid.get_start_pos();
        for mv in movements.iter() {
            pos = perform_movement(&grid, &pos, &mv);
        }
        assert_eq!(pos, (7, 5));
    }

    #[test]
    fn test_password_calculation() {
        let password = calculate_password(include_str!("../../input/day22-test"));
        assert_eq!(password, 6032);
    }
}
