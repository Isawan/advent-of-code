use std::{fmt::Debug, iter::repeat};

use clap::Parser;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::newline,
    combinator::{map, opt, value},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Item {
    Empty,
    ForwardMirror,
    BackMirror,
    VSplit,
    HSplit,
}

impl Default for Item {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Grid<T> {
    items: Vec<T>,
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    left: Grid<bool>,
    right: Grid<bool>,
    up: Grid<bool>,
    down: Grid<bool>,
}

impl<T: Copy + Default> Grid<T> {
    fn new(arrays: Vec<Vec<T>>) -> Self {
        let height = arrays.len() as i32;
        let width = arrays[0].len() as i32;
        let mut items = Vec::with_capacity(((width + 2) * (height + 2)) as usize);
        items.extend(repeat(T::default()).take(width as usize + 2));
        for y in 0..height {
            items.push(T::default());
            items.extend(arrays[y as usize].iter());
            items.push(T::default());
        }
        items.extend(repeat(T::default()).take(width as usize + 2));
        assert_eq!(items.len(), ((width + 2) * (height + 2)) as usize);
        Self {
            items,
            width: width,
            height: height,
        }
    }
}

impl<T: Copy + Debug> Grid<T> {
    fn fresh(width: i32, height: i32, item: T) -> Self {
        let items = vec![item; ((width + 2) * (height + 2)) as usize];
        assert_eq!(items.len(), ((width + 2) * (height + 2)) as usize);
        Self {
            items,
            width,
            height,
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<T> {
        if x < -1 || y < -1 || x > self.width || y > self.height {
            return None;
        }
        let index = (((y + 1) * (self.width + 2)) + (x + 1)) as usize;
        if index >= self.items.len() {}
        Some(self.items[index])
    }

    fn set(&mut self, x: i32, y: i32, item: T) {
        self.items[(((y + 1) * (self.width + 2)) + (x + 1)) as usize] = item;
    }
}

impl<T: Copy + Clone + Debug> Debug for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                match self.get(x, y) {
                    Some(item) => write!(f, "{: <6?}", item)?,
                    None => write!(f, " ")?,
                }
            }
            writeln!(f, "\n")?;
        }
        Ok(())
    }
}

fn grid(input: &str) -> IResult<&str, Grid<Item>> {
    map(
        many1(terminated(
            many1(alt((
                value(Item::Empty, tag(".")),
                value(Item::ForwardMirror, tag("/")),
                value(Item::BackMirror, tag("\\")),
                value(Item::VSplit, tag("|")),
                value(Item::HSplit, tag("-")),
            ))),
            opt(newline),
        )),
        Grid::new,
    )(input)
}

fn update(state: &State, grid: &Grid<Item>) -> State {
    let mut left = Grid::fresh(grid.width, grid.height, false);
    let mut right = Grid::fresh(grid.width, grid.height, false);
    let mut up = Grid::fresh(grid.width, grid.height, false);
    let mut down = Grid::fresh(grid.width, grid.height, false);
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cur_left = state.left.get(x, y).unwrap_or(false);
            let cur_right = state.right.get(x, y).unwrap_or(false);
            let cur_up = state.up.get(x, y).unwrap_or(false);
            let cur_down = state.down.get(x, y).unwrap_or(false);
            match grid.get(x, y) {
                Some(Item::Empty) => {
                    right.set(
                        x,
                        y,
                        cur_right || state.right.get(x - 1, y).unwrap_or(false),
                    );
                    left.set(x, y, cur_left || state.left.get(x + 1, y).unwrap_or(false));
                    up.set(x, y, cur_up || state.up.get(x, y + 1).unwrap_or(false));
                    down.set(x, y, cur_down || state.down.get(x, y - 1).unwrap_or(false));
                }
                Some(Item::ForwardMirror) => {
                    left.set(x, y, cur_left || state.down.get(x, y - 1).unwrap_or(false));
                    right.set(x, y, cur_right || state.up.get(x, y + 1).unwrap_or(false));
                    up.set(x, y, cur_up || state.right.get(x - 1, y).unwrap_or(false));
                    down.set(x, y, cur_down || state.left.get(x + 1, y).unwrap_or(false));
                }
                Some(Item::BackMirror) => {
                    left.set(x, y, cur_left || state.up.get(x, y + 1).unwrap_or(false));
                    right.set(x, y, cur_right || state.down.get(x, y - 1).unwrap_or(false));
                    up.set(x, y, cur_up || state.left.get(x + 1, y).unwrap_or(false));
                    down.set(x, y, cur_down || state.right.get(x - 1, y).unwrap_or(false));
                }
                Some(Item::VSplit) => {
                    let v = state.right.get(x - 1, y).unwrap_or(false)
                        || state.left.get(x + 1, y).unwrap_or(false);
                    up.set(x, y, cur_up || v || state.up.get(x, y + 1).unwrap_or(false));
                    down.set(
                        x,
                        y,
                        cur_down || v || state.down.get(x, y - 1).unwrap_or(false),
                    );
                }
                Some(Item::HSplit) => {
                    let v = state.up.get(x, y + 1).unwrap_or(false)
                        || state.down.get(x, y - 1).unwrap_or(false);
                    left.set(
                        x,
                        y,
                        cur_left || v || state.left.get(x + 1, y).unwrap_or(false),
                    );
                    right.set(
                        x,
                        y,
                        cur_right || v || state.right.get(x - 1, y).unwrap_or(false),
                    );
                }
                None => panic!("outside boundary"),
            }
        }
    }
    State {
        left,
        right,
        up,
        down,
    }
}

fn simulate_until_stable(state: State, grid: &Grid<Item>) -> State {
    let mut state = state;
    loop {
        let new_state = update(&state, grid);
        if new_state == state {
            return state;
        }
        state = new_state;
    }
}

fn count_energized(state: &State) -> u32 {
    let mut count = 0;
    for y in 0..state.left.height {
        for x in 0..state.left.width {
            if state.left.get(x, y).unwrap_or(false)
                || state.right.get(x, y).unwrap_or(false)
                || state.up.get(x, y).unwrap_or(false)
                || state.down.get(x, y).unwrap_or(false)
            {
                count += 1;
            }
        }
    }
    count
}

fn part1(input: &str) -> u32 {
    let grid = grid(input).unwrap().1;
    let mut right = Grid::fresh(grid.width, grid.height, false);
    right.set(-1, 0, true);
    let state = State {
        up: Grid::fresh(grid.width, grid.height, false),
        down: Grid::fresh(grid.width, grid.height, false),
        left: Grid::fresh(grid.width, grid.height, false),
        right,
    };
    let state = simulate_until_stable(state, &grid);
    count_energized(&state)
}

fn part2(input: &str) -> u32 {
    let grid = grid(input).unwrap().1;
    let mut starting_positions: Vec<(i32, i32)> = Default::default();
    for x in 0..grid.width {
        starting_positions.push((x, -1)); // down
        starting_positions.push((x, grid.height)); // up
    }
    for y in 0..grid.height {
        starting_positions.push((-1, y)); // right
        starting_positions.push((grid.width, y)); // left
    }
    starting_positions
        .iter()
        .map(|pos| {
            match pos {
                (x, y) if *y == -1 => State {
                    up: Grid::fresh(grid.width, grid.height, false),
                    down: {
                        let mut g = Grid::fresh(grid.width, grid.height, false);
                        g.set(*x, *y, true);
                        g
                    },
                    left: Grid::fresh(grid.width, grid.height, false),
                    right: Grid::fresh(grid.width, grid.height, false),
                }, // down
                (x, y) if *y == grid.height => State {
                    up: {
                        let mut g = Grid::fresh(grid.width, grid.height, false);
                        g.set(*x, *y, true);
                        g
                    },
                    down: Grid::fresh(grid.width, grid.height, false),
                    left: Grid::fresh(grid.width, grid.height, false),
                    right: Grid::fresh(grid.width, grid.height, false),
                }, // up
                (x, y) if *x == -1 => State {
                    up: Grid::fresh(grid.width, grid.height, false),
                    down: Grid::fresh(grid.width, grid.height, false),
                    left: Grid::fresh(grid.width, grid.height, false),
                    right: {
                        let mut g = Grid::fresh(grid.width, grid.height, false);
                        g.set(*x, *y, true);
                        g
                    },
                }, // right
                (x, y) if *x == grid.width => State {
                    up: Grid::fresh(grid.width, grid.height, false),
                    down: Grid::fresh(grid.width, grid.height, false),
                    left: {
                        let mut g = Grid::fresh(grid.width, grid.height, false);
                        g.set(*x, *y, true);
                        g
                    },
                    right: Grid::fresh(grid.width, grid.height, false),
                }, // left
                (x, y) => panic!("Unexpected initial position. x: {}, y: {}", x, y),
            }
        })
        .map(|s| simulate_until_stable(s, &grid))
        .map(|s| count_energized(&s))
        .max()
        .unwrap()
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    println!("Part 1: {}", part1(&input));
    println!("Part 2: {}", part2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day16_example_one_step() {
        let input = include_str!("../../input/day16-example");
        let grid = grid(input).unwrap().1;
        let mut right = Grid::fresh(grid.width, grid.height, false);
        right.set(0, 0, true);
        let state = State {
            left: Grid::fresh(grid.width, grid.height, false),
            up: Grid::fresh(grid.width, grid.height, false),
            down: Grid::fresh(grid.width, grid.height, false),
            right,
        };
        let state = update(&state, &grid);
        assert_eq!(state.left.get(0, 0), Some(false));
        assert_eq!(state.right.get(0, 0), Some(true));
        assert_eq!(state.up.get(0, 0), Some(false));
        assert_eq!(state.down.get(0, 0), Some(false));

        assert_eq!(state.down.get(1, 0), Some(true));
        assert_eq!(state.up.get(1, 0), Some(true));
    }

    #[test]
    fn test_day16_example() {
        let input = include_str!("../../input/day16-example");
        assert_eq!(part1(input), 46);
    }

    #[test]
    fn test_day16_part2_example() {
        let input = include_str!("../../input/day16-example");
        assert_eq!(part2(input), 51);
    }
}
