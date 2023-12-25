use std::fmt::Debug;

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

impl<T: Copy> Grid<T> {
    fn new(items: Vec<Vec<T>>) -> Self {
        let height = items.len() as i32;
        let width = items[0].len() as i32;
        let items = items.into_iter().flat_map(|x| x.into_iter()).collect();
        Self {
            items,
            width,
            height,
        }
    }

    fn fresh(width: i32, height: i32, item: T) -> Self {
        let items = vec![item; (width * height) as usize];
        Self {
            items,
            width,
            height,
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<T> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some(self.items[(y * self.width + x) as usize])
    }

    fn set(&mut self, x: i32, y: i32, item: T) {
        self.items[((y * self.width) + x) as usize] = item;
    }
}

impl<T: Copy + Clone + Debug> Debug for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //for self.items
        for y in 0..self.height {
            for x in 0..self.width {
                match self.get(x, y) {
                    Some(item) => write!(f, "{: <6?}", item)?,
                    None => write!(f, " ")?,
                }
            }
            writeln!(f)?;
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

fn simulate_until_stable(state: State, grid: &Grid<Item>) -> (u32, State) {
    let mut state = state;
    let mut rounds = 0;
    loop {
        let new_state = update(&state, grid);
        rounds += 1;
        if new_state == state {
            return (rounds, state);
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
    let mut up = Grid::fresh(grid.width, grid.height, false);
    let mut down = Grid::fresh(grid.width, grid.height, false);
    let mut left = Grid::fresh(grid.width, grid.height, false);
    let mut right = Grid::fresh(grid.width, grid.height, false);
    match grid.get(0, 0) {
        Some(Item::Empty) => {
            right.set(0, 0, true);
        }
        Some(Item::ForwardMirror) => {
            up.set(0, 0, true);
        }
        Some(Item::BackMirror) => down.set(0, 0, true),
        Some(Item::VSplit) => {
            up.set(0, 0, true);
            down.set(0, 0, true)
        }
        Some(Item::HSplit) => {
            left.set(0, 0, true);
            right.set(0, 0, true)
        }
        None => panic!("outside boundary"),
    }
    let state = State {
        left,
        right,
        up,
        down,
    };
    let state = simulate_until_stable(state, &grid).1;
    count_energized(&state)
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    println!("Part 1: {}", part1(&input));
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
        println!("{:?}", &state);
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
}
