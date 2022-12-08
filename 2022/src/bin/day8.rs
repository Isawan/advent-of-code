use itertools::Itertools;
use std::cmp::max;
use std::collections::BTreeMap;
use std::iter::zip;
use std::ops::Bound::Excluded;
use std::ops::Bound::Included;
use std::ops::RangeBounds;
use std::ops::RangeFrom;
use structopt::StructOpt;
use itertools::iproduct;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

type Grid = (Vec<u8>, (usize, usize));

fn parse(input: &str) -> (Vec<u8>, (usize, usize)) {
    let result = input
        .replace("\n", "")
        .chars()
        .map(|c| c.to_digit(10).unwrap())
        .map(|u| u as u8)
        .collect();
    let height = input.lines().count();
    let width = input.find("\n").unwrap();
    (result, (width, height))
}

fn print(grid: &Grid) {
    for i in (0..grid.1 .0) {
        for j in (0..grid.1 .1) {
            print!("{}", get(&grid, i.into(), j.into()));
        }
        print!("\n");
    }
}

fn get(grid: &Grid, x: usize, y: usize) -> u8 {
    grid.0[grid.1 .0 * y + x]
}
fn set(grid: &mut Grid, x: usize, y: usize, v: u8) {
    grid.0[grid.1 .0 * y + x] = v
}

fn visible(grid: &Grid) -> Grid {
    let mut result = [
        (vec![0; grid.1 .0 * grid.1 .1], grid.1),
        (vec![0; grid.1 .0 * grid.1 .1], grid.1),
        (vec![0; grid.1 .0 * grid.1 .1], grid.1),
        (vec![0; grid.1 .0 * grid.1 .1], grid.1),
        (vec![0; grid.1 .0 * grid.1 .1], grid.1),
    ];
    for y in 0..(grid.1 .1) {
        let mut tallest_tree = 0;
        let mut visible = true;
        for x in 0..(grid.1 .0) {
            let v = get(grid, x, y);
            visible = tallest_tree < v;
            tallest_tree = max(tallest_tree, v);
            set(&mut result[0], x, y, visible as u8);
        }
    }
    for y in 0..(grid.1 .1) {
        let mut tallest_tree = 0;
        let mut visible = true;
        for x in (0..(grid.1 .0)).rev() {
            let v = get(grid, x, y);
            visible = tallest_tree < v;
            tallest_tree = max(tallest_tree, v);
            set(&mut result[1], x, y, visible as u8);
        }
    }
    for x in (0..(grid.1 .1)) {
        let mut tallest_tree = 0;
        let mut visible = true;
        for y in (0..(grid.1 .0)) {
            let v = get(grid, x, y);
            visible = tallest_tree < v;
            tallest_tree = max(tallest_tree, v);
            set(&mut result[2], x, y, visible as u8);
        }
    }
    for x in (0..(grid.1 .1)) {
        let mut tallest_tree = 0;
        let mut visible = true;
        for y in (0..(grid.1 .0)).rev() {
            let v = get(grid, x, y);
            visible = tallest_tree < v;
            tallest_tree = max(tallest_tree, v);
            set(&mut result[3], x, y, visible as u8);
        }
    }
    for i in 0..(grid.1 .0) {
        set(&mut result[4], i, 0, 1);
        set(&mut result[4], i, (grid.1 .1) - 1, 1);
    }
    for i in 0..(grid.1 .1) {
        set(&mut result[4], 0, i, 1);
        set(&mut result[4], (grid.1 .1) - 1, i, 1);
    }
    let phase = zip(
        zip(result[0].0.iter(), result[1].0.iter()),
        zip(result[2].0.iter(), result[3].0.iter()),
    )
    .map(|((a, b), (c, d))| (a != &0 || b != &0 || c != &0 || d != &0) as u8);
    let out = zip(phase, result[4].0.iter())
        .map(|(a, b)| (a != 0 || b != &0) as u8)
        .collect();
    (out, grid.1)
}

fn scenic_score(grid: &Grid, x: usize, y: usize) -> usize {
    let my_height = get(grid, x, y);
    // go right
    let right = (x..grid.1 .0)
        .skip(1)
        .fold((false, 0), |a, i| {
            match (a.0, get(grid, i, y)) {
                (true, _) => a,
                (false, height) if height >= my_height=> (true, a.1 + 1),
                (false, height) => (false, a.1 + 1),
            }
        })
        .1;
    // go left
    let left = (0..=x)
        .rev()
        .skip(1)
        .fold((false, 0), |a, i| {
            match (a.0, get(grid, i, y)) {
                (true, _) => a,
                (false, height) if height >= my_height=> (true, a.1 + 1),
                (false, height) => (false, a.1 + 1),
            }
        })
        .1;
    // go down
    let down = (y..grid.1 .1)
        .skip(1)
        .fold((false, 0), |a, i| {
            match (a.0, get(grid, x, i)) {
                (true, _) => a,
                (false, height) if height >= my_height=> (true, a.1 + 1),
                (false, height) => (false, a.1 + 1),
            }
        })
        .1;
    // go up
    let up = (0..=y)
        .rev()
        .skip(1)
        .fold((false, 0), |a, i| {
            match (a.0, get(grid, x, i)) {
                (true, _) => a,
                (false, height) if height >= my_height=> (true, a.1 + 1),
                (false, height) => (false, a.1 + 1),
            }
        })
        .1;
    up * down * left * right
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let grid = parse(&input);
    let visibility = visible(&grid);
    println!(
        "visible trees: {}",
        visibility.0.iter().filter(|x| **x == 1).count()
    );
    let mut coords = iproduct!((0..grid.1.0), (0..grid.1.1)).collect::<Vec<(usize,usize)>>();
    coords.sort_by_cached_key(|(j,i)| scenic_score(&grid, *j, *i));
    let (x,y) = coords.iter().last().unwrap();
    println!("x: {}, y: {}, score: {}", x,y,scenic_score(&grid,*x,*y));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_start() {
        let test = include_str!("../../input/day8-test");
        let result = parse(test);
        assert_eq!(result.1 .0, 5);
        assert_eq!(result.1 .1, 5);
    }

    #[test]
    fn test_example() {
        let test = include_str!("../../input/day8-test");
        let result = visible(&parse(test));
        assert_eq!(result.0.iter().filter(|x| **x == 1).count(), 21);
    }

    #[test]
    fn test_scenic() {
        let test = include_str!("../../input/day8-test");
        assert_eq!(scenic_score(&parse(test), 2, 1), 4);
        assert_eq!(scenic_score(&parse(test), 2, 3), 8);
    }
}
