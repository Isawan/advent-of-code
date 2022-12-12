use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
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
    //fn set(grid: &mut Self, x: usize, y: usize, v: u8) {
    //    grid.field[grid.width * y + x] = v
    //}
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
            .map(|c| c & 0b0011_1111) // ascii to number a=1, b=2, etc..
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
                .find(|(i, c)| *c == 'S')
                .map(|(i, _)| i)
                .unwrap(),
        );
        let end = grid.index_to_position(
            input
                .replace("\n", "")
                .as_str()
                .chars()
                .enumerate()
                .find(|(i, c)| *c == 'E')
                .map(|(i, _)| i)
                .unwrap(),
        );
        (grid, start, end)
    }
}

fn search_space() -> &'static [(isize, isize)] {
    &[(1, 0), (0, 1), (-1, 0), (0, -1)]
}

fn move_pos(
    grid: &Grid,
    old_pos: (usize, usize),
    direction: (isize, isize),
) -> Option<((usize, usize), u8)> {
    let old_value = grid.get(old_pos).expect("old position invalid");
    let checked_add = |old: usize, dir: isize| {
        if dir >= 0 {
            old.checked_add(dir as usize)
        } else {
            old.checked_sub((-dir) as usize)
        }
    };

    match (
        checked_add(old_pos.0, direction.0),
        checked_add(old_pos.1, direction.1),
    ) {
        (Some(x), Some(y)) => {
            if let Some(new_value) = grid.get((x, y)) {
                if new_value - 1 <= old_value {
                    Some(((x, y), new_value))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn search(grid: &Grid, start: (usize, usize), end: (usize, usize)) -> Option<StepCount> {
    let mut candidates = BinaryHeap::new();
    let mut previous_positions = BTreeSet::new();
    let mut distance = 0;
    let value = grid.get(start).expect("start position not on the board?");
    let minimum_distance;
    candidates.push((Reverse(distance), start));
    previous_positions.insert(start);

    loop {
        let (distance, pos) = candidates.pop()?;
        let value = grid.get(pos).unwrap();

        if let Some((new_pos, new_value)) = move_pos(&grid, pos, (1, 0)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push((Reverse(distance.0 + 1), new_pos));
            }
        }
        if let Some((new_pos, new_value)) = move_pos(&grid, pos, (0, 1)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push((Reverse(distance.0 + 1), new_pos));
            }
        }
        if let Some((new_pos, new_value)) = move_pos(&grid, pos, (-1, 0)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push((Reverse(distance.0 + 1), new_pos));
            }
        }
        if let Some((new_pos, new_value)) = move_pos(&grid, pos, (0, -1)) {
            if !previous_positions.contains(&new_pos) {
                previous_positions.insert(new_pos);
                candidates.push((Reverse(distance.0 + 1), new_pos));
            }
        }
        if pos == end {
            minimum_distance = distance.0;
            break;
        }
    }
    Some(minimum_distance)
}

fn search_lowest_exhaust(grid: &Grid, end: (usize, usize)) -> StepCount {
    let mut minimum_positions = Vec::new();
    let minimum_value = grid.field.iter().fold(u8::MAX, |a,x| std::cmp::min(a, *x));
    println!("minimum: {}", minimum_value);
    for i in 0..grid.width {
        for j in 0..grid.height {
            if let Some(value) = grid.get((i, j)) {
                if (i,j) == end {continue}
                if value == minimum_value {
                    minimum_positions.push((i, j));
                }
            }
        }
    }
    minimum_positions
        .iter()
        .filter_map(|pos| search(&grid, *pos, end))
        .fold(u32::MAX, std::cmp::min)
}

// fn search_lowest(grid: Grid, start: (usize, usize)) -> StepCount {
//     let mut candidates = BinaryHeap::new();
//     let mut previous_positions = BTreeSet::new();
//     let mut distance = 0;
//     let value = grid.get(start).expect("start position not on the board?");
//     let minimum_distance;
//     candidates.push((Reverse(distance), start));
//     previous_positions.insert(start);
//
//     loop {
//         let (distance, pos) = candidates.pop().expect("exhausted search");
//         println!("{:?}", pos);
//         let value = grid.get(pos).unwrap();
//
//         if let Some((new_pos, new_value)) = move_down_pos(&grid, pos, (1, 0)) {
//             if !previous_positions.contains(&new_pos) {
//                 previous_positions.insert(new_pos);
//                 candidates.push((Reverse(distance.0 + 1), new_pos));
//             }
//         }
//         if let Some((new_pos, new_value)) = move_down_pos(&grid, pos, (0, 1)) {
//             if !previous_positions.contains(&new_pos) {
//                 previous_positions.insert(new_pos);
//                 candidates.push((Reverse(distance.0 + 1), new_pos));
//             }
//         }
//         if let Some((new_pos, new_value)) = move_down_pos(&grid, pos, (-1, 0)) {
//             if !previous_positions.contains(&new_pos) {
//                 previous_positions.insert(new_pos);
//                 candidates.push((Reverse(distance.0 + 1), new_pos));
//             }
//         }
//         if let Some((new_pos, new_value)) = move_down_pos(&grid, pos, (0, -1)) {
//             if !previous_positions.contains(&new_pos) {
//                 previous_positions.insert(new_pos);
//                 candidates.push((Reverse(distance.0 + 1), new_pos));
//             }
//         }
//         match grid.get(pos) {
//             Some(v) if (v == 1) => {return distance.0;},
//             _ => {},
//         }
//     }
//     minimum_distance
// }

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let (grid, start_pos, end_pos) = Grid::parse(&input);
    println!("{:?}", search(&grid, start_pos, end_pos));
    println!("{:?}", search_lowest_exhaust(&grid, end_pos));

    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use std::iter::Inspect;

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
        assert_eq!(search(&grid, start, end), 31);
    }

    #[test]
    fn test_search_back() {
        let input = include_str!("../../input/day12-test");
        let (grid, start, end) = Grid::parse(input);
        assert_eq!(search_lowest_exhaust(&grid, end), 29);
    }
}
