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
}

struct StateOrd(State);

impl PartialOrd for StateOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.0.distance.cmp(&self.0.distance))
    }
}

impl Ord for StateOrd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.0.distance.cmp(&self.0.distance)
    }
}

impl PartialEq for StateOrd {
    fn eq(&self, other: &Self) -> bool {
        self.0.distance == other.0.distance
    }
}

impl Eq for StateOrd {}

fn traverse(grid: HashMap<(i32, i32), u32>) -> Option<u32> {
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
    }));
    let mut visited: HashSet<State> = HashSet::default();
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
        if visited.contains(&state) {
            continue;
        }
        if (x, y) == destination {
            return Some(distance);
        }
        let next_directions = match direction {
            Direction::Start => vec![
                Direction::Left(2),
                Direction::Right(2),
                Direction::Up(2),
                Direction::Down(2),
            ],
            Direction::Down(0) => vec![Direction::Left(2), Direction::Right(2)],
            Direction::Down(n) => vec![
                Direction::Left(2),
                Direction::Right(2),
                Direction::Down(n - 1),
            ],
            Direction::Up(0) => vec![Direction::Left(2), Direction::Right(2)],
            Direction::Up(n) => vec![
                Direction::Left(2),
                Direction::Right(2),
                Direction::Up(n - 1),
            ],
            Direction::Left(0) => vec![Direction::Up(2), Direction::Down(2)],
            Direction::Left(n) => {
                vec![Direction::Up(2), Direction::Down(2), Direction::Left(n - 1)]
            }
            Direction::Right(0) => vec![Direction::Up(2), Direction::Down(2)],
            Direction::Right(n) => vec![
                Direction::Up(2),
                Direction::Down(2),
                Direction::Right(n - 1),
            ],
        };
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
                }));
            }
        }
        visited.insert(state);
    }
    None
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(&args.path).unwrap();
    let grid = parse_grid(&input);
    println!("Part 1: {}", traverse(grid).unwrap());
    // println!("Part 2: {}", grid.len());
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
        });
        let state2 = StateOrd(State {
            x: 0,
            y: 0,
            distance: 1,
            direction: Direction::Left(0),
        });
        assert!(state2 < state1);
    }

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day17-example");
        let grid = parse_grid(input);
        assert_eq!(traverse(grid), Some(102));
    }
}
