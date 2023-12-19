use std::collections::{BinaryHeap, HashMap, HashSet};

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

// parse grid of numbers
fn parse_grid(input: &str) -> HashMap<(i32, i32), u32> {
    let mut grid = HashMap::new();
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            grid.insert((x as i32, y as i32), c.to_digit(10).unwrap());
        }
    }
    grid
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Left(u32),
    Right(u32),
    Up(u32),
    Down(u32),
    Start,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    x: i32,
    y: i32,
    distance: u32,
    direction: Direction,
    destination: (i32, i32),
}

struct StateOrd(State);

impl PartialOrd for StateOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((other.0.distance + other.heuristic()).cmp(&(self.0.distance + self.heuristic())))
    }
}

impl Ord for StateOrd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (other.0.distance + other.heuristic()).cmp(&(self.0.distance + self.heuristic()))
    }
}

impl PartialEq for StateOrd {
    fn eq(&self, other: &Self) -> bool {
        (self.0.distance + self.heuristic()) == (other.0.distance + other.heuristic())
    }
}

impl StateOrd {
    fn heuristic(&self) -> u32 {
        let Self(State {
            x,
            y,
            destination: (dest_x, dest_y),
            ..
        }) = self;
        (x - dest_x).unsigned_abs() + (y - dest_y).unsigned_abs()
    }
}

impl Eq for StateOrd {}

fn part1_state_generator(s: &State) -> Vec<Direction> {
    match s.direction {
        Direction::Start => vec![
            Direction::Left(1),
            Direction::Right(1),
            Direction::Up(1),
            Direction::Down(1),
        ],
        Direction::Down(3..) => vec![Direction::Left(1), Direction::Right(1)],
        Direction::Down(n) => vec![
            Direction::Left(1),
            Direction::Right(1),
            Direction::Down(n + 1),
        ],
        Direction::Up(3..) => vec![Direction::Left(1), Direction::Right(1)],
        Direction::Up(n) => vec![
            Direction::Left(1),
            Direction::Right(1),
            Direction::Up(n + 1),
        ],
        Direction::Left(3..) => vec![Direction::Up(1), Direction::Down(1)],
        Direction::Left(n) => {
            vec![Direction::Up(1), Direction::Down(1), Direction::Left(n + 1)]
        }
        Direction::Right(3..) => vec![Direction::Up(1), Direction::Down(1)],
        Direction::Right(n) => vec![
            Direction::Up(1),
            Direction::Down(1),
            Direction::Right(n + 1),
        ],
    }
}

fn part2_state_generator(s: &State) -> Vec<Direction> {
    match s.direction {
        Direction::Start => vec![
            Direction::Left(1),
            Direction::Right(1),
            Direction::Up(1),
            Direction::Down(1),
        ],
        Direction::Down(10..) => vec![Direction::Left(1), Direction::Right(1)],
        Direction::Down(n @ ..=3) => vec![Direction::Down(n + 1)],
        Direction::Down(n @ 4..=9) => vec![
            Direction::Left(1),
            Direction::Right(1),
            Direction::Down(n + 1),
        ],
        Direction::Up(10..) => vec![Direction::Left(1), Direction::Right(1)],
        Direction::Up(n @ ..=3) => vec![Direction::Up(n + 1)],
        Direction::Up(n @ 4..=9) => vec![
            Direction::Left(1),
            Direction::Right(1),
            Direction::Up(n + 1),
        ],
        Direction::Left(10..) => vec![Direction::Up(1), Direction::Down(1)],
        Direction::Left(n @ ..=3) => vec![Direction::Left(n + 1)],
        Direction::Left(n @ 4..=9) => {
            vec![Direction::Up(1), Direction::Down(1), Direction::Left(n + 1)]
        }
        Direction::Right(10..) => vec![Direction::Up(1), Direction::Down(1)],
        Direction::Right(n @ ..=3) => vec![Direction::Right(n + 1)],
        Direction::Right(n @ 4..=9) => vec![
            Direction::Up(1),
            Direction::Down(1),
            Direction::Right(n + 1),
        ],
    }
}

fn traverse(
    grid: &HashMap<(i32, i32), u32>,
    generator: impl Fn(&State) -> Vec<Direction>,
) -> Option<u32> {
    let mut states = BinaryHeap::new();
    let destination = (
        *grid.keys().map(|(x, _)| x).max().unwrap(),
        *grid.keys().map(|(_, y)| y).max().unwrap(),
    );
    states.push(StateOrd(State {
        x: 0,
        y: 0,
        distance: 0,
        direction: Direction::Start,
        destination,
    }));
    let mut visited: HashSet<(i32, i32, Direction)> = HashSet::default();
    while let Some(StateOrd(
        state @ State {
            x,
            y,
            distance,
            direction,
            ..
        },
    )) = states.pop()
    {
        if visited.contains(&(x, y, direction)) {
            continue;
        }
        if (x, y) == destination {
            return Some(distance);
        }
        let next_directions = generator(&state);
        for next_direction in next_directions.into_iter() {
            let new_x = match next_direction {
                Direction::Left(_) => x - 1,
                Direction::Right(_) => x + 1,
                _ => x,
            };
            let new_y = match next_direction {
                Direction::Up(_) => y - 1,
                Direction::Down(_) => y + 1,
                _ => y,
            };
            if let Some(value) = grid.get(&(new_x, new_y)) {
                states.push(StateOrd(State {
                    x: new_x,
                    y: new_y,
                    distance: distance + value,
                    direction: next_direction,
                    destination,
                }));
            }
        }
        visited.insert((x, y, direction));
    }
    None
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    let grid = parse_grid(&input);
    println!(
        "Part 1: {}",
        traverse(&grid, part1_state_generator).unwrap()
    );

    println!(
        "Part 2: {}",
        traverse(&grid, part2_state_generator).unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_ordering() {
        let state1 = StateOrd(State {
            x: 0,
            y: 0,
            distance: 0,
            direction: Direction::Left(0),
            destination: (1, 1),
        });
        let state2 = StateOrd(State {
            x: 0,
            y: 0,
            distance: 1,
            direction: Direction::Left(0),
            destination: (1, 1),
        });
        assert!(state2 < state1);
    }

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day17-example");
        let grid = parse_grid(input);
        assert_eq!(traverse(&grid, part1_state_generator), Some(102));
    }

    #[test]
    fn test_example_part2() {
        let input = include_str!("../../input/day17-example");
        let grid = parse_grid(input);
        assert_eq!(traverse(&grid, part2_state_generator), Some(94));
    }
}
