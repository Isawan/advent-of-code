use std::cell::RefCell;

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

#[derive(Clone, PartialEq, Eq)]
struct Grid {
    items: Vec<Item>,
    width: usize,
    height: usize,
}

impl Grid {
    fn new(items: Vec<Vec<Item>>) -> Self {
        let height = items.len();
        let width = items[0].len();
        let items = items.into_iter().flat_map(|x| x.into_iter()).collect();
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
        Some(self.items[y * self.width + x])
    }

    fn update(self, new_rounds: &mut Vec<(usize, usize)>) -> Self {
        let mut items = self.items;
        // reset grid
        for item in items.iter_mut() {
            *item = match item {
                Item::Round => Item::Empty,
                Item::Cube => Item::Cube,
                Item::Empty => Item::Empty,
            };
        }
        // update grid
        for (x, y) in new_rounds {
            items[*y * self.width + *x] = Item::Round;
        }
        Self {
            items,
            width: self.width,
            height: self.height,
        }
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //for self.items
        //for line in self.items.iter() {
        //    for item in line.iter() {
        //        match item {
        //            Item::Round => write!(f, "O")?,
        //            Item::Cube => write!(f, "#")?,
        //            Item::Empty => write!(f, ".")?,
        //        }
        //    }
        //    writeln!(f)?;
        //}
        Ok(())
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

fn new_round_positions(visits: impl Iterator<Item = (Item)>) -> impl Iterator<Item = usize> {
    visits
        .enumerate()
        .scan(0, move |highest, (w, item)| match item {
            Item::Empty => Some(None),
            Item::Round => {
                *highest += 1;
                Some(Some(*highest - 1))
            }
            Item::Cube => {
                *highest = w + 1;
                Some(None)
            }
        })
        .flatten()
}

fn calc_load(grid: &Grid) -> usize {
    (0..grid.width)
        .flat_map(|x| {
            new_round_positions((0..grid.height).map(move |y| grid.get(x, y).unwrap()))
                .map(move |y| (x, y))
        })
        .map(|(_, y)| grid.height - y)
        .sum()
}

thread_local! {
    static BUF: RefCell<Vec<(usize, usize)>> = RefCell::new(Vec::new());
}

fn cycle(mut grid: Grid) -> Grid {
    BUF.with(|buf| {
        let buf = &mut *buf.borrow_mut();
        grid = {
            let grid_ref = &grid;
            buf.extend((0..grid.width).flat_map(|x| {
                new_round_positions((0..grid.height).map(move |y| grid_ref.get(x, y).unwrap()))
                    .map(move |y| (x, y))
            }));
            grid.update(buf)
        };
        buf.clear();
        grid = {
            let grid_ref = &grid;
            buf.extend((0..grid.height).flat_map(|y| {
                new_round_positions((0..grid.width).map(move |x| grid_ref.get(x, y).unwrap()))
                    .map(move |x| (x, y))
            }));
            grid.update(buf)
        };
        buf.clear();
        grid = {
            let grid_ref = &grid;
            buf.extend((0..grid.width).flat_map(|x| {
                new_round_positions(
                    ((0..grid.height).rev()).map(move |y| grid_ref.get(x, y).unwrap()),
                )
                .map(move |y| (x, grid_ref.height - y - 1))
            }));
            grid.update(buf)
        };
        buf.clear();
        grid = {
            let grid_ref = &grid;
            buf.extend((0..grid.height).flat_map(|y| {
                new_round_positions(
                    ((0..grid.width).rev()).map(move |x| grid_ref.get(x, y).unwrap()),
                )
                .map(move |x| (grid_ref.width - x - 1, y))
            }));
            grid.update(buf)
        };
        buf.clear();
        grid
    })
}

fn part2(input: Grid) -> usize {
    let mut grid = input;
    for i in 0..1_000_000 {
        if i % 100_000 == 0 {
            println!("{}", i);
        }
        grid = cycle(grid);
    }
    let grid_ref = &grid;
    (0..grid.width)
        .flat_map(|x| (0..grid.height).map(move |y| (x, y, grid_ref.get(x, y).unwrap())))
        .filter(|(_, _, item)| *item == Item::Round)
        .map(|(_, y, _)| grid.height - y)
        .sum()
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    println!("Part 1: {}", calc_load(&grid(&input)));
    println!("Part 2: {}", part2(grid(&input)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day14-example");
        let grid = grid(input);
        assert_eq!(calc_load(&grid), 136);
    }

    #[test]
    fn test_example_after_one_cycle() {
        let input = include_str!("../../input/day14-example");
        let one_cycle = grid(include_str!("../../input/day14-example-one-cycle"));
        let grid = grid(input);
        assert_eq!(cycle(grid), one_cycle);
    }

    #[test]
    fn test_example_part2() {
        let input = include_str!("../../input/day14-example");
        let grid = grid(input);
        assert_eq!(part2(grid), 64);
    }
}
