use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Round,
    Cube,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid {
    items: Vec<Vec<Item>>,
    width: usize,
    height: usize,
}

impl Grid {
    fn new(items: Vec<Vec<Item>>) -> Self {
        let height = items.len();
        let width = items[0].len();
        Self {
            items,
            width,
            height,
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<Item> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.items[y][x])
    }
}

fn grid(input: &str) -> Grid {
    Grid::new(
        input
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| match c {
                        '.' => Item::Empty,
                        'O' => Item::Round,
                        '#' => Item::Cube,
                        _ => unreachable!(),
                    })
                    .collect()
            })
            .collect(),
    )
}

fn new_round_positions(grid: &Grid) -> Vec<(usize, usize)> {
    (0..grid.width)
        .flat_map(|x| {
            (0..grid.height)
                .scan(0, move |highest, y| {
                    let item = grid.get(x, y).unwrap();
                    match item {
                        Item::Empty => Some(None),
                        Item::Round => {
                            *highest = *highest + 1;
                            Some(Some((x, *highest - 1)))
                        }
                        Item::Cube => {
                            *highest = y + 1;
                            Some(None)
                        }
                    }
                })
                .filter_map(|x| x)
        })
        .collect()
}

fn part1(grid: &Grid) -> usize {
    new_round_positions(grid)
        .iter()
        .map(|(_, y)| grid.height - y)
        .sum()
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    println!("Part 1: {}", part1(&grid(&input)));
}

#[cfg(test)]
mod tests {
    use super::*;

    //    #[test]
    //    fn test_new_positions() {
    //        let input = include_str!("../../input/day14-example");
    //        let grid = grid(input);
    //        assert_eq!(
    //            new_round_positions,
    //        )
    //    }

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day14-example");
        let grid = grid(input);
        assert_eq!(part1(&grid), 136);
    }
}
