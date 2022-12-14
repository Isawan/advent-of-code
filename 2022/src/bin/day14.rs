use std::cmp::{max, min};
use std::collections::BTreeMap;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug)]
enum Item {
    Wall,
    Sand,
}

type Positions = BTreeMap<(i32, i32), Item>;

fn parse_paths(lines: &str) -> Vec<Vec<(i32, i32)>> {
    lines
        .lines()
        .map(|line| {
            line.split(" -> ")
                .map(|coord| {
                    let mut c = coord.split(",");
                    match (c.next(), c.next()) {
                        (Some(x), Some(y)) => {
                            (x.parse::<i32>().unwrap(), y.parse::<i32>().unwrap())
                        }
                        _ => panic!("unexpected"),
                    }
                })
                .collect::<Vec<(i32, i32)>>()
        })
        .collect::<Vec<Vec<(i32, i32)>>>()
}

fn paths_to_walls(paths: Vec<Vec<(i32, i32)>>) -> Positions{
    let mut positions = BTreeMap::new();
    for path in paths.iter() {
        let mut path_points = path.iter();
        if let Some(mut previous) = path_points.next() {
            while let Some(mut next) = path_points.next() {
                if previous.0 - next.0 == 0 {
                    let x = previous.0;
                    let start = min(previous.1, next.1);
                    let end = max(previous.1, next.1);
                    for y in start..=end {
                        positions.insert((x, y), Item::Wall);
                    }
                } else {
                    let y = previous.1;
                    let start = min(previous.0, next.0);
                    let end = max(previous.0, next.0);
                    for x in start..=end {
                        positions.insert((x, y), Item::Wall);
                    }
                }

                previous = next;
            }
        }
    }
    positions
}

fn move_sand(positions: &Positions, sand: (i32, i32)) -> Option<(i32, i32)> {
    match positions.get(&(sand.0, sand.1 + 1)) {
        None => return Some((sand.0, sand.1 + 1)),
        _ => {}
    };
    match positions.get(&(sand.0 - 1, sand.1 + 1)) {
        None => return Some((sand.0 - 1, sand.1 + 1)),
        _ => {}
    };
    match positions.get(&(sand.0 + 1, sand.1 + 1)) {
        None => return Some((sand.0 + 1, sand.1 + 1)),
        _ => {}
    };
    None
}

fn sim_round(mut positions: Positions, endless: bool) -> (Positions, Option<(i32, i32)>) {
    let bottom = positions.keys().fold(0, |a, x| std::cmp::max(a, x.1));
    let mut sand = (500, 0);
    while let Some(next_sand) = move_sand(&positions, sand) {
        if endless && (next_sand.1 > bottom) {
            return (positions, None);
        }
        sand = next_sand
    }
    positions.insert(sand, Item::Sand);
    (positions, Some(sand))
}

fn add_floor(mut positions: Positions) -> Positions {
    let max_x = positions.keys().fold(i32::MIN, |a, x| max(a, x.0)) + 1000;
    let min_x = positions.keys().fold(i32::MAX, |a, x| min(a, x.0)) - 1000;
    let floor_y = positions.keys().fold(0, |a, x| std::cmp::max(a, x.1)) + 2;
    for x in min_x..=max_x {
        positions.insert((x, floor_y), Item::Wall);
    }
    positions
}

fn count_rounds(input: &str) -> u32 {
    let mut positions = paths_to_walls(parse_paths(input));
    let mut count_rounds = 0;
    loop {
        if let (new_positions, Some(_)) = sim_round(positions, true) {
            positions = new_positions;
        } else {
            break;
        }
        count_rounds = count_rounds + 1;
    }
    count_rounds
}

fn count_until_full(input: &str) -> u32 {
    let mut positions = paths_to_walls(parse_paths(input));
    positions = add_floor(positions);
    let mut count_rounds = 0;
    loop {
        let (new_positions, next) = sim_round(positions, false);
        count_rounds = count_rounds + 1;
        if let Some(_) = new_positions.get(&(500, 0)) {
            break;
        }
        positions = new_positions;
    }
    count_rounds
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    println!("solution 1: {}", count_rounds(&input));
    println!("solution 2: {}", count_until_full(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = include_str!("../../input/day14-test");
        let p = parse_paths(input);
        assert_eq!(
            p,
            vec![
                vec![(498, 4), (498, 6), (496, 6)],
                vec![(503, 4), (502, 4), (502, 9), (494, 9)]
            ]
        );
    }

    #[test]
    fn test_sim_round() {
        let input = include_str!("../../input/day14-test");
        let positions = paths_to_walls(parse_paths(input));
        if let (_, Some(sand)) = sim_round(positions, true) {
            assert_eq!(sand, (500, 8));
        } else {
            panic!("unexpected")
        }
    }

    #[test]
    fn test_to_round_22() {
        let input = include_str!("../../input/day14-test");
        let mut positions = paths_to_walls(parse_paths(input));
        for _ in 0..21 {
            (positions, _) = sim_round(positions, true);
        }
        if let (_, Some(sand)) = sim_round(positions, true) {
            assert_eq!(sand, (500, 2));
        } else {
            panic!("unexpected")
        }
    }

    #[test]
    fn test_to_end() {
        let input = include_str!("../../input/day14-test");
        assert_eq!(count_rounds(input), 24);
    }

    #[test]
    fn test_count_until_full() {
        let input = include_str!("../../input/day14-test");
        assert_eq!(count_until_full(input), 93);
    }
}
