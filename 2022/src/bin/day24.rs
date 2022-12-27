use std::{cmp::min, collections::BinaryHeap, convert::TryInto, time::Instant};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Clone, Debug, Copy)]
enum BlizzardDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
enum Tile {
    Blizzard { direction: BlizzardDirection },
    Ground,
}

impl From<Tile> for char {
    fn from(tile: Tile) -> Self {
        match tile {
            Tile::Ground => '.',
            Tile::Blizzard {
                direction: BlizzardDirection::Down,
            } => 'v',
            Tile::Blizzard {
                direction: BlizzardDirection::Up,
            } => '^',
            Tile::Blizzard {
                direction: BlizzardDirection::Right,
            } => '>',
            Tile::Blizzard {
                direction: BlizzardDirection::Left,
            } => '<',
        }
    }
}

struct Valley {
    field: Vec<Tile>,
    width: i32,
    height: i32,
}

#[derive(Clone, Copy, Debug)]
enum HistoryTile {
    Blizzard,
    Ground,
    Wall,
}

// A view of the valley at future history
struct ValleyHistory {
    blizzard_map: Vec<HistoryTile>,
    width: i32,
    height: i32,
    full_cycle_length: i32,
}

impl ValleyHistory {
    fn new(valley: Valley) -> Self {
        let width = valley.width;
        let height = valley.height;
        let full_cycle_length = width * height;
        let mut blizzard_map =
            vec![HistoryTile::Ground; (full_cycle_length * height * width) as usize];
        let to_index = |x: i32, y: i32, t: i32| -> usize {
            ((t * height * width) + (y.rem_euclid(height) * width) + x.rem_euclid(width))
                .try_into()
                .unwrap()
        };
        for ((start_x, start_y), direction) in valley.blizzards() {
            for t in 0..full_cycle_length {
                match direction {
                    BlizzardDirection::Right => {
                        blizzard_map[to_index(start_x + t, start_y, t)] = HistoryTile::Blizzard;
                    }
                    BlizzardDirection::Left => {
                        blizzard_map[to_index(start_x - t, start_y, t)] = HistoryTile::Blizzard;
                    }
                    BlizzardDirection::Up => {
                        blizzard_map[to_index(start_x, start_y - t, t)] = HistoryTile::Blizzard;
                    }
                    BlizzardDirection::Down => {
                        blizzard_map[to_index(start_x, start_y + t, t)] = HistoryTile::Blizzard;
                    }
                }
            }
        }
        ValleyHistory {
            blizzard_map,
            height,
            width,
            full_cycle_length,
        }
    }

    fn get(&self, position: &(i32, i32), time: i32) -> HistoryTile {
        let (x, y) = *position;
        // we assume the blizzard never covers the ending squares.
        if position == &(0, -1) || position == &(self.width - 1, self.height) {
            return HistoryTile::Ground;
        }
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return HistoryTile::Wall;
        }
        let index: usize = (self.width * self.height * time.rem_euclid(self.full_cycle_length)
            + (self.width * y)
            + x)
            .try_into()
            .unwrap();
        self.blizzard_map[index]
    }

    #[allow(dead_code)]
    fn display_at_time(&self, time: i32) -> String {
        let mut s = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                s.push(match self.get(&(x, y), time) {
                    HistoryTile::Blizzard => 'O',
                    HistoryTile::Ground => '.',
                    HistoryTile::Wall => unreachable!(),
                })
            }
            s.push('\n');
        }
        s
    }
}

impl From<Valley> for ValleyHistory {
    fn from(valley: Valley) -> Self {
        ValleyHistory::new(valley)
    }
}

impl Valley {
    fn blizzards(&self) -> impl Iterator<Item = ((i32, i32), BlizzardDirection)> + '_ {
        let width = self.width;
        self.field.iter().enumerate().filter_map(move |(i, tile)| {
            let x = (i as i32).rem_euclid(width);
            let y = (i as i32).div_euclid(width);
            match tile {
                Tile::Blizzard { direction } => Some(((x, y), *direction)),
                _ => None,
            }
        })
    }
}

fn parse(input: &str) -> Valley {
    let original_height = input.lines().count() as i32;
    let original_width = input.find('\n').unwrap() as i32;
    let new_width = original_width - 2;
    let new_height = original_height - 2;
    let mut field = vec![Tile::Ground; ((original_height - 2) * (original_width - 2)) as usize];
    let _no_newline_input = input.replace("\n", "");
    let visit = _no_newline_input.as_str().chars().enumerate();
    for (i, c) in visit {
        let original_x = (i as i32).rem_euclid(original_width);
        let original_y = (i as i32).div_euclid(original_width);
        if original_x == 0
            || original_y == 0
            || original_x == original_width - 1
            || original_y == original_height - 1
        {
            // we don't want the edges (we'll handle this within the ValleyHistory datastructure)
            continue;
        }
        let new_x = original_x - 1;
        let new_y = original_y - 1;
        let new_index = <i32 as TryInto<usize>>::try_into((new_width * new_y) + new_x).unwrap();
        field[new_index] = match c {
            '>' => Tile::Blizzard {
                direction: BlizzardDirection::Right,
            },
            '<' => Tile::Blizzard {
                direction: BlizzardDirection::Left,
            },
            'v' => Tile::Blizzard {
                direction: BlizzardDirection::Down,
            },
            '^' => Tile::Blizzard {
                direction: BlizzardDirection::Up,
            },
            '.' => Tile::Ground,
            _ => panic!("unexpected character"),
        }
    }
    Valley {
        field: field,
        width: new_width,
        height: new_height,
    }
}

fn adjacent(centre: &(i32, i32)) -> impl Iterator<Item = (i32, i32)> + '_ {
    [(0, 0), (1, 0), (0, 1), (-1, 0), (0, -1)]
        .iter()
        .map(move |x| (centre.0 + x.0, centre.1 + x.1))
}

#[derive(Debug, PartialEq, Eq, Ord)]
struct Ranker<'a> {
    position: (i32, i32),
    target: &'a (i32, i32),
    time: i32,
}

impl Ranker<'_> {
    fn distance_to_target(&self) -> i32 {
        let (x, y) = self.position;
        let (end_x, end_y) = self.target;
        (end_x - x).abs() + (end_y - y).abs()
    }
}

impl PartialOrd for Ranker<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // other -> self. order matters because we want a min-heap
        other
            .distance_to_target()
            .partial_cmp(&self.distance_to_target())
    }
}

fn get_start() -> (i32, i32) {
    (0, -1)
}
fn get_end(history: &ValleyHistory) -> (i32, i32) {
    (history.width - 1, history.height)
}

fn search(
    history: &ValleyHistory,
    start_time: i32,
    segment: ((i32, i32), (i32, i32)),
) -> Option<i32> {
    let (start, target) = segment;
    let mut queue = BinaryHeap::with_capacity(1_000_000);
    let height = history.height;
    let width = history.width;
    let mut visited = vec![false; (width * height * history.full_cycle_length) as usize];

    // use cyclic condition to avoid repeats
    let to_index = |x: i32, y: i32, t: i32| -> usize {
        ((t.rem_euclid(history.full_cycle_length) * height * width)
            + (y.rem_euclid(height) * width)
            + x.rem_euclid(width))
        .try_into()
        .unwrap()
    };
    let mut best = None;
    let mut candidates = Vec::with_capacity(5);
    queue.push(Ranker {
        position: start,
        target: &target,
        time: start_time,
    });
    while let Some(
        rank @ Ranker {
            position,
            target,
            time,
        },
    ) = queue.pop()
    {
        if &position == target {
            best = best.or(Some(time)).map(|best_time| min(best_time, time));
            continue;
        }

        // bound all branches that are worst than the best found case.
        if let Some(best_time) = best {
            if best_time < time || best_time < time + rank.distance_to_target() {
                continue;
            }
        }

        candidates.extend(
            adjacent(&position)
                .filter(|adj| !visited[to_index(adj.0, adj.1, time + 1)])
                .filter_map(|adj| match history.get(&adj, time + 1) {
                    HistoryTile::Ground => Some(Ranker {
                        position: adj,
                        target,
                        time: time + 1,
                    }),
                    HistoryTile::Blizzard => None,
                    HistoryTile::Wall => None,
                }),
        );
        for rank @ Ranker {
            position: (next_x, next_y),
            ..
        } in candidates.drain(..)
        {
            visited[to_index(next_x, next_y, time + 1)] = true;
            queue.push(rank);
        }
    }
    best
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    let history = ValleyHistory::new(parse(&input));
    println!(
        "solution 1: {:?}",
        search(&history, 0, (get_start(), get_end(&history)))
    );
    println!(
        "solution 2: {:?}",
        search(&history, 0, (get_start(), get_end(&history)))
            .and_then(|time| search(&history, time, (get_end(&history), get_start())))
            .and_then(|time| search(&history, time, (get_start(), get_end(&history)))),
    );
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let _ = parse(include_str!("../../input/day24-test"));
    }

    #[test]
    fn test_valley_history() {
        let valley = parse(include_str!("../../input/day24-test"));
        let _ = ValleyHistory::new(valley);
    }

    #[test]
    fn test_example() {
        let history = ValleyHistory::new(parse(include_str!("../../input/day24-test")));
        assert_eq!(
            search(&history, 0, (get_start(), get_end(&history))),
            Some(18)
        );
    }

    #[test]
    fn test_multiple_segments() {
        let history = ValleyHistory::new(parse(include_str!("../../input/day24-test")));
        assert_eq!(
            search(&history, 0, (get_start(), get_end(&history)))
                .and_then(|time| search(&history, time, (get_end(&history), get_start())))
                .and_then(|time| search(&history, time, (get_start(), get_end(&history)))),
            Some(54)
        );
    }
}
