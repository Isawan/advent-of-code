use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::Iterator;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
enum NonStartedPipe {
    Start,
    Pipe(Vec<(i32, i32)>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ScanState {
    Outside,
    Inside,
    UpOut,
    UpIn,
    DownOut,
    DownIn,
}

impl ScanState {
    pub(crate) fn is_inside(&self) -> bool {
        matches!(self, ScanState::Inside)
    }

    pub(crate) fn next(&self, c: char) -> Self {
        match (self, c) {
            (ScanState::Outside, '|') => ScanState::Inside,
            (ScanState::Outside, 'F') => ScanState::DownOut,
            (ScanState::Outside, 'L') => ScanState::UpOut,
            (ScanState::Inside, '|') => ScanState::Outside,
            (ScanState::Inside, 'F') => ScanState::DownIn,
            (ScanState::Inside, 'L') => ScanState::UpIn,
            (ScanState::DownOut, '7') => ScanState::Outside,
            (ScanState::DownOut, 'J') => ScanState::Inside,
            (ScanState::DownIn, '7') => ScanState::Inside,
            (ScanState::DownIn, 'J') => ScanState::Outside,
            (ScanState::UpOut, '7') => ScanState::Inside,
            (ScanState::UpOut, 'J') => ScanState::Outside,
            (ScanState::UpIn, '7') => ScanState::Outside,
            (ScanState::UpIn, 'J') => ScanState::Inside,
            (s, '-') => *s,
            _ => panic!("Invalid state transition: {:?} {}", self, c),
        }
    }
}

fn pipe(tile: char) -> NonStartedPipe {
    match tile {
        '|' => NonStartedPipe::Pipe(vec![(0, 1), (0, -1)]),
        '-' => NonStartedPipe::Pipe(vec![(-1, 0), (1, 0)]),
        'L' => NonStartedPipe::Pipe(vec![(1, 0), (0, -1)]),
        'J' => NonStartedPipe::Pipe(vec![(-1, 0), (0, -1)]),
        '7' => NonStartedPipe::Pipe(vec![(0, 1), (-1, 0)]),
        'F' => NonStartedPipe::Pipe(vec![(0, 1), (1, 0)]),
        '.' => NonStartedPipe::Pipe(vec![]),
        'S' => NonStartedPipe::Start,
        _ => panic!("Invalid tile: {}", tile),
    }
}

fn resolve_start(
    mut pipes: HashMap<(i32, i32), Vec<(i32, i32)>>,
    start: (i32, i32),
) -> HashMap<(i32, i32), Vec<(i32, i32)>> {
    let start_adjacent_pipes: Vec<(i32, i32)> = pipes
        .iter()
        .filter_map(|(k, v)| if v.contains(&start) { Some(k) } else { None })
        .cloned()
        .collect();
    for pipe in start_adjacent_pipes.iter() {
        pipes.entry(start).or_default().push(*pipe);
    }
    pipes
}

#[allow(clippy::type_complexity)]
fn build_pipe_map(input: &str) -> ((i32, i32), HashMap<(i32, i32), Vec<(i32, i32)>>) {
    let mut pipes: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
    let mut start = None;
    for (y, line) in input.lines().enumerate() {
        for (x, tile) in line.chars().enumerate() {
            let pipe = pipe(tile);
            match pipe {
                NonStartedPipe::Start => {
                    start = Some((x as i32, y as i32));
                }
                NonStartedPipe::Pipe(adjacent) => {
                    if !adjacent.is_empty() {
                        pipes.entry((x as i32, y as i32)).or_default().extend(
                            adjacent
                                .iter()
                                .map(|(dx, dy)| ((x as i32) + dx, (y as i32) + dy)),
                        );
                    }
                }
            }
        }
    }
    let start = start.expect("No start found");
    (start, resolve_start(pipes, start))
}

fn find_furthest_point(map: HashMap<(i32, i32), Vec<(i32, i32)>>, start: (i32, i32)) -> u32 {
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut explore = VecDeque::new();
    let mut max_steps = 0;
    explore.push_back((0, start));
    while let Some((steps, current)) = explore.pop_front() {
        let adjacent = map.get(&current).unwrap();
        visited.insert(current);
        explore.extend(
            adjacent
                .iter()
                .filter(|adjacent| !visited.contains(adjacent))
                .map(|adjacent| (steps + 1, *adjacent)),
        );
        max_steps = max_steps.max(steps);
    }
    max_steps
}

fn count_enclosed_area_by_loop(
    map: HashMap<(i32, i32), Vec<(i32, i32)>>,
    start: (i32, i32),
) -> u32 {
    let mut new_map = HashMap::new();
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut explore = VecDeque::new();

    // get max y and x
    let max_y = *map.keys().map(|(_, y)| y).max().unwrap();
    let max_x = *map.keys().map(|(x, _)| x).max().unwrap();

    // prune map to only include pipe attached to start
    explore.push_back(start);
    while let Some(current) = explore.pop_front() {
        let adjacent = map.get(&current).unwrap();
        visited.insert(current);
        explore.extend(
            adjacent
                .iter()
                .filter(|adjacent| !visited.contains(adjacent))
                .cloned(),
        );
        new_map.insert(current, adjacent.clone());
    }
    let map = new_map;

    // scan map for enclosed area with modified odd-even rule
    // Special state machine to handle edge cases where corner is hit
    let mut inside = HashSet::new();
    for j in 0..=max_y {
        let mut ray_state = ScanState::Outside;
        for i in 0..=max_x {
            if let Some(kv) = map.get_key_value(&(i, j)) {
                ray_state = ray_state.next(pipe_to_char(kv));
            } else if ray_state.is_inside() {
                inside.insert((i, j));
            }
        }
    }
    inside.len() as u32
}

fn pipe_to_char(pipe: (&(i32, i32), &Vec<(i32, i32)>)) -> char {
    let ((pipe_x, pipe_y), adjacent) = pipe;
    let delta = adjacent
        .iter()
        .map(|(x, y)| (x - pipe_x, y - pipe_y))
        .collect::<Vec<_>>();

    match delta.as_slice() {
        [(0, 1), (0, -1)] | [(0, -1), (0, 1)] => '|',
        [(-1, 0), (1, 0)] | [(1, 0), (-1, 0)] => '-',
        [(1, 0), (0, -1)] | [(0, -1), (1, 0)] => 'L',
        [(-1, 0), (0, -1)] | [(0, -1), (-1, 0)] => 'J',
        [(0, 1), (-1, 0)] | [(-1, 0), (0, 1)] => '7',
        [(0, 1), (1, 0)] | [(1, 0), (0, 1)] => 'F',
        _ => panic!("Invalid pipe: {:?}", delta),
    }
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    let (start, map) = build_pipe_map(&input);
    println!("Part 1: {}", find_furthest_point(map.clone(), start));
    println!(
        "Part 2: {}",
        count_enclosed_area_by_loop(map.clone(), start)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_furthest_point_example_1() {
        let input = include_str!("../../input/day10-example-1");
        let (start, map) = build_pipe_map(input);
        assert_eq!(find_furthest_point(map, start), 4);
    }

    #[test]
    fn test_furthest_point_example_2() {
        let input = include_str!("../../input/day10-example-2");
        let (start, map) = build_pipe_map(input);
        assert_eq!(find_furthest_point(map, start), 8);
    }

    #[test]
    fn test_enclosed_area_example_1() {
        let input = include_str!("../../input/day10-example-1");
        let (start, map) = build_pipe_map(input);
        assert_eq!(count_enclosed_area_by_loop(map, start), 1);
    }

    #[test]
    fn test_enclosed_area_example_3() {
        let input = include_str!("../../input/day10-example-3");
        let (start, map) = build_pipe_map(input);
        assert_eq!(count_enclosed_area_by_loop(map, start), 4);
    }

    #[test]
    fn test_enclosed_area_example_4() {
        let input = include_str!("../../input/day10-example-4");
        let (start, map) = build_pipe_map(input);
        assert_eq!(count_enclosed_area_by_loop(map, start), 8);
    }
}
