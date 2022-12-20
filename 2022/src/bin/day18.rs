use std::cmp::{max, min};
use std::collections::HashSet;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

type Point = (i32, i32, i32);

fn parse(input: &str) -> HashSet<Point> {
    input
        .lines()
        .map(|line| {
            let mut coords = line.split(',');
            (
                coords.next().unwrap().parse().unwrap(),
                coords.next().unwrap().parse().unwrap(),
                coords.next().unwrap().parse().unwrap(),
            )
        })
        .collect::<HashSet<Point>>()
}

fn adjacent(p: &Point) -> impl Iterator<Item = Point> + '_ {
    [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ]
    .iter()
    .map(move |x| (p.0 + x.0, p.1 + x.1, p.2 + x.2))
}

fn search_exposed(points: &HashSet<Point>, p: &(i32, i32, i32)) -> u32 {
    adjacent(p).filter(|x| !points.contains(x)).count() as u32
}

fn area(points: &HashSet<Point>) -> u32 {
    points.iter().map(|p| search_exposed(&points, p)).sum()
}

fn find_corners(points: &HashSet<Point>) -> (Point, Point) {
    let max_x = points.iter().map(|x| x.0).fold(0, max) + 1;
    let max_y = points.iter().map(|x| x.1).fold(0, max) + 1;
    let max_z = points.iter().map(|x| x.2).fold(0, max) + 1;
    let min_x = points.iter().map(|x| x.0).fold(i32::MAX, min) - 1;
    let min_y = points.iter().map(|x| x.1).fold(i32::MAX, min) - 1;
    let min_z = points.iter().map(|x| x.2).fold(i32::MAX, min) - 1;
    ((min_x, min_y, min_z), (max_x, max_y, max_z))
}

fn in_bounds(p: &Point, bounds: &(Point, Point)) -> bool {
    p.0 >= min(bounds.0 .0, bounds.1 .0)
        && p.0 <= max(bounds.0 .0, bounds.1 .0)
        && p.1 >= min(bounds.0 .1, bounds.1 .1)
        && p.1 <= max(bounds.0 .1, bounds.1 .1)
        && p.2 >= min(bounds.0 .2, bounds.1 .2)
        && p.2 <= max(bounds.0 .2, bounds.1 .2)
}

fn outside(droplet: &HashSet<Point>) -> HashSet<Point> {
    let bounds = find_corners(droplet);
    let start = bounds.0;
    let mut search = vec![start];
    let mut visited: HashSet<Point> = HashSet::new();
    let mut next_search = Vec::new();
    while let Some(point) = search.pop() {
        next_search.extend(
            adjacent(&point)
                .filter(|p| in_bounds(p, &bounds))
                .filter(|p| !droplet.contains(p))
                .filter(|p| !&visited.contains(p)),
        );
        visited.extend(next_search.iter());
        search.append(&mut next_search);
    }
    visited
}

fn search_external(droplet: &HashSet<Point>) -> u32 {
    let outside_points = outside(droplet);
    droplet
        .iter()
        .map(|p| adjacent(p).filter(|p| outside_points.contains(p)).count() as u32)
        .sum()
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    let points = parse(&input);
    println!("solution 1: {}", area(&points));
    println!("solution 2: {}", search_external(&points));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let input = parse(include_str!("../../input/day18-test"));
        assert_eq!(input.len(), 13);
        assert!(input.contains(&(2, 2, 2)));
        assert!(input.contains(&(1, 2, 2)));
    }

    #[test]
    fn test_search_adjacent() {
        let mut input = HashSet::new();
        input.insert((1, 1, 1));
        input.insert((2, 1, 1));
        assert_eq!(search_exposed(&input, &(2, 1, 1)), 5);
        assert_eq!(search_exposed(&input, &(1, 1, 1)), 5);
    }

    #[test]
    fn test_example() {
        let input = parse(include_str!("../../input/day18-test"));
        assert_eq!(area(&input), 64);
    }

    #[test]
    fn test_outside_detection() {
        let droplet = parse(include_str!("../../input/day18-test"));
        let outside = outside(&droplet);
        assert!(!outside.contains(&(2, 2, 5)));
        assert!(!outside
            .union(&droplet)
            .copied()
            .collect::<HashSet<Point>>()
            .contains(&(2, 2, 5)));
        assert_eq!(outside.len() + droplet.len() + 1, 200);
    }

    #[test]
    fn test_search_external() {
        let droplet = parse(include_str!("../../input/day18-test"));
        assert_eq!(search_external(&droplet), 58);
    }
}
