use std::cmp::Reverse;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct Grid {
    field: Vec<u8>,
    width: usize,
    height: usize,
}

type Start = (usize, usize);
type End = (usize, usize);
type StepCount = u32;

impl Grid {
    fn get(&self, pos: (usize, usize)) -> Option<u8> {
        if pos.0 >= self.width {
            return None;
        }
        if pos.1 >= self.height {
            return None;
        }
        Some(self.field[self.width * pos.1 + pos.0])
    }

    fn index_to_position(&self, i: usize) -> (usize, usize) {
        (i % self.width, i / self.width)
    }

    fn parse(input: &str) -> (Self, Start, End) {
        let height = input.lines().count();
        let width = input.find('\n').unwrap();
        let field = input
            .replace("\n", "")
            .as_str()
            .as_bytes()
            .iter()
            .map(|c| {
                if *c == ('S' as u8) {
                    'a' as u8
                } else if *c == ('E' as u8) {
                    'z' as u8
                } else {
                    *c
                }
            }) // handle start and end position
            .map(|c| c & 0b0001_1111) // ascii to number a=1, b=2, etc..
            .collect();
        let grid = Grid {
            field,
            width,
            height,
        };
        let start = grid.index_to_position(
            input
                .replace("\n", "")
                .as_str()
                .chars()
                .enumerate()
                .find(|(_, c)| *c == 'S')
                .map(|(i, _)| i)
                .unwrap(),
        );
        let end = grid.index_to_position(
            input
                .replace("\n", "")
                .as_str()
                .chars()
                .enumerate()
                .find(|(_, c)| *c == 'E')
                .map(|(i, _)| i)
                .unwrap(),
        );
        (grid, start, end)
    }
}

fn add(old: usize, dir: isize) -> Option<usize> {
    if dir.is_negative() {
        old.checked_sub(dir.wrapping_abs() as usize)
    } else {
        old.checked_add(dir as usize)
    }
}

fn climb_up(
    grid: &Grid,
    old_pos: (usize, usize),
    direction: (isize, isize),
) -> Option<(usize, usize)> {
    let old_value = grid.get(old_pos).expect("old position invalid");
    add(old_pos.0, direction.0)
        .zip(add(old_pos.1, direction.1))
        .and_then(|p| grid.get(p).map(|new_value| (p, new_value)))
        .filter(|(_, new_value)| new_value - 1 <= old_value)
        .map(|x| x.0)
}

fn climb_down(
    grid: &Grid,
    old_pos: (usize, usize),
    direction: (isize, isize),
) -> Option<(usize, usize)> {
    let old_value = grid.get(old_pos).expect("old position invalid");
    add(old_pos.0, direction.0)
        .zip(add(old_pos.1, direction.1))
        .and_then(|p| grid.get(p).map(|new_value| (p, new_value)))
        .filter(|(_, new_value)| old_value - 1 <= *new_value)
        .map(|x| x.0)
}

fn end_at(end_pos: (usize, usize)) -> impl Fn(&Grid, (usize, usize)) -> bool {
    move |_, x| end_pos == x
}

fn end_when_meet(c: u8) -> impl Fn(&Grid, (usize, usize)) -> bool {
    move |grid, p| {
        let v = grid.get(p).expect("expected real");
        v == c
    }
}

fn search(
    grid: &Grid,
    start: (usize, usize),
    valid_move: impl Fn(&Grid, (usize, usize), (isize, isize)) -> Option<(usize, usize)>,
    end_condition: impl Fn(&Grid, (usize, usize)) -> bool,
) -> Option<StepCount> {
    let mut candidates = VecDeque::new();
    let mut previous_positions = HashSet::new();
    let distance = 0;
    candidates.push_back((Reverse(distance), start));
    previous_positions.insert(start);

    loop {
        let (Reverse(distance), pos) = candidates.pop_front()?;

        if let Some(new_pos) = valid_move(&grid, pos, (1, 0)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push_back((Reverse(distance + 1), new_pos));
            }
        }
        if let Some(new_pos) = valid_move(&grid, pos, (0, 1)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push_back((Reverse(distance + 1), new_pos));
            }
        }
        if let Some(new_pos) = valid_move(&grid, pos, (-1, 0)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push_back((Reverse(distance + 1), new_pos));
            }
        }
        if let Some(new_pos) = valid_move(&grid, pos, (0, -1)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push_back((Reverse(distance + 1), new_pos));
            }
        }

        if end_condition(grid, pos) {
            return Some(distance);
        }
    }
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let (grid, start_pos, end_pos) = Grid::parse(&input);
    println!("width: {}, height: {}", grid.width, grid.height);
    println!(
        "found S: {:?}",
        search(&grid, start_pos, climb_up, end_at(end_pos)).expect("no path found")
    );
    println!(
        "found a: {:?}",
        search(&grid, end_pos, climb_down, end_when_meet(1)).expect("no path found")
    );

    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let input = include_str!("../../input/day12-test");
        Grid::parse(input);
    }

    #[test]
    fn test_search() {
        let input = include_str!("../../input/day12-test");
        let (grid, start, end) = Grid::parse(input);
        assert_eq!(search(&grid, start, climb_up, end_at(end)), Some(31));
    }

    #[test]
    fn test_move() {
        let input = include_str!("../../input/day12-test");
        let (grid, _, _) = Grid::parse(input);
        assert_eq!(climb_up(&grid, (0, 2), (0, 1)), Some((0, 3)));
        assert_eq!(climb_up(&grid, (0, 2), (-1, 0)), None);
        assert_eq!(climb_up(&grid, (2, 2), (0, 1)), Some((2, 3)));
        assert_eq!(climb_up(&grid, (2, 3), (1, 0)), None);
    }

    #[test]
    fn test_climb_down() {
        let input = include_str!("../../input/day12-test");
        let (grid, _, _) = Grid::parse(input);
        assert_eq!(climb_down(&grid, (5, 2), (0, -1)), None);
        assert_eq!(climb_down(&grid, (5, 2), (0, 1)), None);
        assert_eq!(climb_down(&grid, (5, 2), (-1, 0)), Some((4, 2)));
        assert_eq!(climb_down(&grid, (5, 2), (1, 0)), None);

        assert_eq!(climb_down(&grid, (2, 2), (0, -1)), Some((2, 1)));
        assert_eq!(climb_down(&grid, (2, 2), (0, 1)), Some((2, 3)));
        assert_eq!(climb_down(&grid, (2, 2), (-1, 0)), Some((1, 2)));
        assert_eq!(climb_down(&grid, (2, 2), (1, 0)), Some((3, 2)));

        assert_eq!(climb_down(&grid, (5, 1), (0, -1)), None);
        assert_eq!(climb_down(&grid, (5, 1), (0, 1)), Some((5, 2)));
        assert_eq!(climb_down(&grid, (5, 1), (-1, 0)), Some((4, 1)));
        assert_eq!(climb_down(&grid, (5, 1), (1, 0)), Some((6, 1)));

        assert_eq!(climb_down(&grid, (4, 2), (-1, 0)), None);
        assert_eq!(climb_down(&grid, (4, 2), (1, 0)), Some((5, 2)));

        assert_eq!(climb_down(&grid, (3, 2), (-1, 0)), None);
        assert_eq!(climb_down(&grid, (3, 2), (1, 0)), Some((4, 2)));

        assert_eq!(climb_down(&grid, (2, 2), (-1, 0)), Some((1, 2)));
        assert_eq!(climb_down(&grid, (2, 2), (1, 0)), Some((3, 2)));

        assert_eq!(climb_down(&grid, (1, 2), (-1, 0)), None);
        assert_eq!(climb_down(&grid, (1, 2), (1, 0)), Some((2, 2)));

        assert_eq!(climb_down(&grid, (0, 2), (-1, 0)), None);
        assert_eq!(climb_down(&grid, (0, 2), (1, 0)), Some((1, 2)));
    }

    #[test]
    fn test_search_until() {
        let input = include_str!("../../input/day12-test");
        let (grid, _, end) = Grid::parse(input);
        assert_eq!(search(&grid, end, climb_down, end_when_meet(1)), Some(29));
    }
}
