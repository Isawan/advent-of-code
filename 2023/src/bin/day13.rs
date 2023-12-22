use clap::Parser;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::{map, map_res, opt, value},
    multi::many1,
    sequence::terminated,
    IResult,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Ash,
    Rock,
}

struct Grid {
    items: Vec<Vec<Item>>,
    width: i32,
    height: i32,
}

impl Grid {
    fn new(items: Vec<Vec<Item>>) -> Self {
        let height = items.len() as i32;
        let width = items[0].len() as i32;
        Self {
            items,
            width,
            height,
        }
    }
    fn get(&self, x: i32, y: i32) -> Option<Item> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some(self.items[y as usize][x as usize])
    }
}

// parse a grid of characters
fn grid(input: &str) -> IResult<&str, Grid> {
    map(
        many1(terminated(
            many1(alt((
                value(Item::Ash, tag(".")),
                value(Item::Rock, tag("#")),
            ))),
            opt(newline),
        )),
        Grid::new,
    )(input)
}

fn grids(input: &str) -> IResult<&str, Vec<Grid>> {
    many1(terminated(grid, opt(newline)))(input)
}

fn flip_x((x, y): (i32, i32), x_axis: i32) -> (i32, i32) {
    (2 * x_axis - x - 1, y)
}

fn flip_y((x, y): (i32, i32), y_axis: i32) -> (i32, i32) {
    (x, 2 * y_axis - y - 1)
}

fn horizontal_flip(grid: &Grid) -> Option<i32> {
    // enumerate over all cell in grid
    (1..grid.width).find(|axis| {
        grid.items
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, item)| ((x, y), item)))
            .map(|((x, y), item)| ((x, y), flip_x((x as i32, y as i32), *axis), item))
            .map(|((x, y), (fx, fy), item)| (item, grid.get(fx, fy)))
            .all(|(item, other)| match (item, other) {
                (_, None) => true,
                (first, Some(flipped)) => *first == flipped,
            })
    })
}

fn vertical_flip(grid: &Grid) -> Option<i32> {
    // enumerate over all cell in grid
    (1..grid.height).find(|axis| {
        grid.items
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, item)| ((x, y), item)))
            .map(|((x, y), item)| ((x, y), flip_y((x as i32, y as i32), *axis), item))
            .map(|((x, y), (fx, fy), item)| (item, grid.get(fx, fy)))
            .all(|(item, other)| match (item, other) {
                (_, None) => true,
                (first, Some(flipped)) => *first == flipped,
            })
    })
}

fn part1(input: &str) -> i32 {
    let grids = grids(input).unwrap().1;
    grids
        .iter()
        .map(|grid| {
            horizontal_flip(grid).unwrap_or_else(|| vertical_flip(grid).map(|v| 100 * v).unwrap())
        })
        .sum()
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(&args.path).unwrap();
    println!("Part 1: {}", part1(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_x() {
        assert_eq!(flip_x((0, 0), 5), (9, 0));
        assert_eq!(flip_x((1, 0), 5), (8, 0));
        assert_eq!(flip_x((2, 0), 5), (7, 0));
        assert_eq!(flip_x((3, 0), 5), (6, 0));
        assert_eq!(flip_x((4, 0), 5), (5, 0));
        assert_eq!(flip_x((5, 0), 5), (4, 0));
        assert_eq!(flip_x((6, 0), 5), (3, 0));
        assert_eq!(flip_x((7, 0), 5), (2, 0));
        assert_eq!(flip_x((8, 0), 5), (1, 0));
    }

    #[test]
    fn test_flip_y() {
        assert_eq!(flip_y((0, 0), 5), (0, 9));
        assert_eq!(flip_y((0, 1), 5), (0, 8));
        assert_eq!(flip_y((0, 2), 5), (0, 7));
        assert_eq!(flip_y((0, 3), 5), (0, 6));
        assert_eq!(flip_y((0, 4), 5), (0, 5));
        assert_eq!(flip_y((0, 5), 5), (0, 4));
        assert_eq!(flip_y((0, 6), 5), (0, 3));
        assert_eq!(flip_y((0, 7), 5), (0, 2));
        assert_eq!(flip_y((0, 8), 5), (0, 1));
    }

    #[test]
    fn test_parse_day13() {
        let input = include_str!("../../input/day13-example");
        let grids = grids(input).unwrap().1;
        assert_eq!(grids.len(), 2);
    }

    #[test]
    fn test_example_horizontal_flip() {
        let input = include_str!("../../input/day13-example");
        let grids = grids(input).unwrap().1;
        let grid = &grids[0];
        assert_eq!(horizontal_flip(grid), Some(5));
    }

    #[test]
    fn test_example_vertical_flip() {
        let input = include_str!("../../input/day13-example");
        let grids = grids(input).unwrap().1;
        let grid = &grids[1];
        assert_eq!(vertical_flip(grid), Some(4));
    }

    #[test]
    fn test_example_part1() {
        let input = include_str!("../../input/day13-example");
        assert_eq!(part1(input), 405);
    }
}
