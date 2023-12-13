use std::{
    cmp::min,
    collections::{btree_map::Range, BTreeSet, HashSet},
    ops::Bound,
    u128::MAX,
};

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

// grid parse
fn expand_galaxy(input: &str, expansion_factor: usize) -> BTreeSet<(i64, i64)> {
    let mut original_grid = BTreeSet::new();
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            match c {
                '#' => {
                    original_grid.insert((x, y));
                }
                '.' => {}
                _ => panic!("Invalid char: {}", c),
            }
        }
    }

    let max_x = original_grid.iter().map(|(x, _)| *x).max().unwrap();
    let max_y = original_grid.iter().map(|(_, y)| *y).max().unwrap();

    let all_x = original_grid
        .iter()
        .map(|(x, _)| *x)
        .collect::<BTreeSet<usize>>();
    let all_y = original_grid
        .iter()
        .map(|(_, y)| *y)
        .collect::<BTreeSet<usize>>();

    let empty_x = (0..max_x)
        .filter(|x| !all_x.contains(x))
        .collect::<BTreeSet<usize>>();
    let empty_y = (0..max_y)
        .filter(|y| !all_y.contains(y))
        .collect::<BTreeSet<usize>>();

    original_grid
        .into_iter()
        .map(|(x, y)| {
            (
                (x + (expansion_factor - 1)
                    * empty_x
                        .range((Bound::Unbounded, Bound::Excluded(x)))
                        .count()) as i64,
                (y + (expansion_factor - 1)
                    * empty_y
                        .range((Bound::Unbounded, Bound::Excluded(y)))
                        .count()) as i64,
            )
        })
        .collect()
}

fn sum_of_neighbour_distance(input: &str, factor: usize) -> i64 {
    let map = expand_galaxy(input, factor);
    let mut total = 0;
    for left in map.iter() {
        for right in map.range((Bound::Excluded(left), Bound::Unbounded)) {
            let (lx, ly) = left;
            let (rx, ry) = right;
            let distance = (lx - rx).abs() + (ly - ry).abs();
            total += distance;
        }
    }
    total
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    println!("Part 1: {}", sum_of_neighbour_distance(&input, 2));
    println!("Part 2: {}", sum_of_neighbour_distance(&input, 1_000_000));
}

mod tests {
    use super::*;

    #[test]
    fn test_expand_galaxy() {
        let input = include_str!("../../input/day11-example-1");
        let output = expand_galaxy(input, 2);
        println!("{:?}", output);
        assert_eq!(output.len(), 9);
        assert!(output.contains(&(4, 0)));
        assert!(output.contains(&(9, 1)));
    }

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day11-example-1");
        assert_eq!(sum_of_neighbour_distance(&input, 2), 374);
    }

    #[test]
    fn test_example_with_expansion_ten() {
        let input = include_str!("../../input/day11-example-1");
        assert_eq!(sum_of_neighbour_distance(&input, 10), 1030);
    }

    #[test]
    fn test_example_with_expansion_one_hundred() {
        let input = include_str!("../../input/day11-example-1");
        let output = expand_galaxy(input, 100);
        assert_eq!(sum_of_neighbour_distance(&input, 100), 8410);
    }
}
