use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(PartialEq, Clone, Debug)]
enum Error {
    OutOfBounds { i: i32, j: i32 },
}

fn parse_input(source: &str) -> HashMap<(i32, i32), u32> {
    let mut result = HashMap::new();
    for (j, line) in source.split('\n').enumerate() {
        for (i, c) in line.chars().enumerate() {
            result.insert((i as i32, j as i32), c.to_digit(10).unwrap());
        }
    }
    result
}
fn is_low(grid: &HashMap<(i32, i32), u32>, i: i32, j: i32) -> Result<bool, Error> {
    if let Some(value) = grid.get(&(i, j)) {
        let minimum_neighbour = &[(i, j - 1), (i, j + 1), (i - 1, j), (i + 1, j)]
            .iter()
            .filter_map(|(k, l)| grid.get(&(*k, *l)))
            .fold(u32::MAX, |a, x| cmp::min(a, *x));
        return Ok(minimum_neighbour > value);
    } else {
        return Err(Error::OutOfBounds { i, j });
    }
}

fn stream_lowpoints(grid: &HashMap<(i32, i32), u32>) -> Vec<u32> {
    grid.keys()
        .filter_map(|(i, j)| {
            if is_low(grid, *i, *j).unwrap() {
                grid.get(&(*i, *j))
            } else {
                None
            }
        })
        .map(|x| x.clone())
        .collect()
}

fn lowpoints_coords(grid: &HashMap<(i32, i32), u32>) -> Vec<(i32, i32)> {
    grid.keys()
        .filter_map(|(i, j)| {
            if is_low(grid, *i, *j).unwrap() {
                Some((*i, *j))
            } else {
                None
            }
        })
        .map(|x| x.clone())
        .collect()
}

struct SearchState {
    pos: (i32, i32),
}

fn basin_size(grid: &HashMap<(i32, i32), u32>, i: i32, j: i32) -> u32 {
    assert!(is_low(grid, i, j).is_ok());
    let mut visited = HashSet::<(i32, i32)>::new();
    let mut search_stack = Vec::new();
    let mut basin_count = 0;
    search_stack.push(SearchState { pos: (i, j) });
    loop {
        if let Some(state) = search_stack.pop() {
            if visited.contains(&state.pos) {
                continue;
            }
            let (x, y) = (state.pos.0, state.pos.1);
            let neighbours = &[(x, y - 1), (x, y + 1), (x - 1, y), (x + 1, y)];
            visited.insert((x, y));
            let value = grid.get(&(x, y)).unwrap();
            // handle saddlepoints, ignore for now
            //let count_lower = neighbours
            //    .iter()
            //    .filter_map(|(k, l)| grid.get(&(*k, *l)))
            //    .filter(|v| *v < value)
            //    .count();
            //if count_lower > 1 {
            //    continue;
            //}

            // search neighbour
            for (k, l) in neighbours {
                if let Some(neighbour_value) = grid.get(&(*k, *l)) {
                    if *neighbour_value != 9 && neighbour_value >= &value {
                        search_stack.push(SearchState { pos: (*k, *l) })
                    }
                }
            }
            basin_count += 1;
        } else {
            break;
        }
    }
    basin_count
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let grid = parse_input(&source);
    let sum_risk_level = stream_lowpoints(&grid)
        .iter()
        .map(|x| x + 1)
        .fold(0, |a, x| a + x);
    println!("sum of risk level: {}", sum_risk_level);
    let mut basins = lowpoints_coords(&grid)
        .iter()
        .map(|(i, j)| basin_size(&grid, *i, *j))
        .collect::<Vec<u32>>();
    basins.sort();
    let x = basins[basins.len() - 3..].iter().fold(1, |a, x| a * x);
    println!("{}", x);
}
#[cfg(test)]
mod tests {
    use super::*;

    fn test_parse_input() {
        let input = parse_input("222\n212\n331\n");
        assert_eq!(input.get(&(0, 0)), Some(&2));
        assert_eq!(input.get(&(0, 1)), Some(&2));
        assert_eq!(input.get(&(0, 2)), Some(&2));
        assert_eq!(input.get(&(1, 0)), Some(&2));
        assert_eq!(input.get(&(1, 1)), Some(&1));
        assert_eq!(input.get(&(1, 2)), Some(&2));
        assert_eq!(input.get(&(2, 0)), Some(&3));
        assert_eq!(input.get(&(2, 1)), Some(&3));
        assert_eq!(input.get(&(2, 2)), Some(&2));
    }

    fn test_is_low() {
        let mut input = HashMap::new();
        input.insert((0, 0), 2);
        input.insert((0, 1), 2);
        input.insert((0, 2), 2);
        input.insert((1, 0), 2);
        input.insert((1, 1), 1);
        input.insert((1, 2), 2);
        input.insert((2, 0), 3);
        input.insert((2, 1), 3);
        input.insert((2, 2), 2);
        assert_eq!(is_low(&input, 1, 1), Ok(true));
        assert_eq!(is_low(&input, 2, 1), Ok(false));
    }

    #[test]
    fn test_basin_size_simple() {
        let mut input = HashMap::new();
        input.insert((0, 0), 2);
        input.insert((0, 1), 2);
        input.insert((0, 2), 2);
        input.insert((1, 0), 2);
        input.insert((1, 1), 1);
        input.insert((1, 2), 2);
        input.insert((2, 0), 3);
        input.insert((2, 1), 3);
        input.insert((2, 2), 2);
        let size = basin_size(&input, 1, 1);
        assert_eq!(size, 9);
    }
    #[test]
    fn test_basin_size_valley() {
        let mut input = HashMap::new();
        input.insert((0, 0), 2);
        input.insert((0, 1), 1);
        input.insert((0, 2), 2);
        input.insert((1, 0), 2);
        input.insert((1, 1), 2);
        input.insert((1, 2), 2);
        input.insert((2, 0), 9);
        input.insert((2, 1), 9);
        input.insert((2, 2), 9);
        input.insert((3, 0), 2);
        input.insert((3, 1), 2);
        input.insert((3, 2), 2);
        input.insert((4, 0), 2);
        input.insert((4, 1), 1);
        input.insert((4, 2), 2);
        let size = basin_size(&input, 0, 1);
        assert_eq!(size, 6);
    }
}
