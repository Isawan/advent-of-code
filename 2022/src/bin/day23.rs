use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    fmt,
    time::Instant,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive()]
struct Elf<'a> {
    state: ElfState,
    rules_order: [(&'a [(i32, i32)], (i32, i32)); 4],
}

impl<'a> fmt::Debug for Elf<'a> {
    fn fmt<'b>(&self, f: &mut fmt::Formatter<'b>) -> fmt::Result {
        write!(f, "Elf {{ {:?} }}", self.state)
    }
}

impl Elf<'_> {
    fn new() -> Self {
        Elf {
            state: ElfState::Noop,
            rules_order: [
                (&[(-1, -1), (0, -1), (1, -1)], (0, -1)),
                (&[(-1, 1), (0, 1), (1, 1)], (0, 1)),
                (&[(-1, -1), (-1, 0), (-1, 1)], (-1, 0)),
                (&[(1, -1), (1, 0), (1, 1)], (1, 0)),
            ],
        }
    }

    fn to_noop(mut self) -> Self {
        self.state = ElfState::Noop;
        self
    }

    // initially I thought the question required a different rule set per elf depending on whether
    // which rule gets used. turns out it was way simpler than expected.
    fn propose(mut self, elves_position: &HashSet<(i32, i32)>, old_pos: &(i32, i32)) -> Self {
        let new_pos = self
            .rules_order
            .iter()
            .enumerate()
            .find_map(|(i, r)| rule(r.0, r.1)(elves_position, old_pos).map(|v| (i, v)));
        match new_pos {
            None => self.to_noop(),
            Some((i, p)) => {
                self.state = ElfState::ProposedMove {
                    pos: p,
                    rule_index: i,
                };
                self
            }
        }
    }
}

#[derive(Clone, Debug)]
enum ElfState {
    ProposedMove { rule_index: usize, pos: (i32, i32) },
    Noop,
}

fn parse(input: &str) -> HashMap<(i32, i32), Elf> {
    let width = input.find('\n').unwrap();
    input
        .replace("\n", "")
        .as_str()
        .chars()
        .enumerate()
        .filter_map(|(i, c)| match c {
            '.' => None,
            '#' => Some((
                (i.rem_euclid(width) as i32, i.div_euclid(width) as i32),
                Elf::new(),
            )),
            _ => panic!("unexpected character"),
        }) // handle start and end position
        .collect()
}

fn adjacent(centre: &(i32, i32)) -> impl Iterator<Item = (i32, i32)> + '_ {
    [
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ]
    .iter()
    .map(move |x| (centre.0 + x.0, centre.1 + x.1))
}

fn first_half(elves: HashMap<(i32, i32), Elf>) -> HashMap<(i32, i32), Elf> {
    // dirty clone to get around lifetime rules in the apply stage
    let old_elves_position: HashSet<(i32, i32)> = elves.keys().cloned().collect();
    elves
        .into_par_iter()
        .map(|(pos, elf)| {
            if adjacent(&pos)
                .filter(|adj| old_elves_position.contains(&adj))
                .count()
                .eq(&0)
            {
                (pos.clone(), elf.to_noop())
            } else {
                (pos.clone(), elf.propose(&old_elves_position, &pos))
            }
        })
        .collect()
}

fn second_half(elves: HashMap<(i32, i32), Elf>) -> HashMap<(i32, i32), Elf> {
    let counter = elves
        .values()
        .fold(HashMap::new(), |mut counter, elf| match elf.state {
            ElfState::Noop => counter,
            ElfState::ProposedMove {
                pos: pos,
                rule_index: _,
            } => {
                *counter.entry(pos).or_insert(0) += 1;
                counter
            }
        });

    elves
        .into_par_iter()
        .map(|(pos, mut elf)| {
            elf.rules_order.rotate_left(1);
            match elf.state.clone() {
                ElfState::Noop => (pos, elf.to_noop()),
                ElfState::ProposedMove {
                    pos: new_pos,
                    rule_index: index,
                } => {
                    if *counter.get(&new_pos).unwrap() == 1 {
                        (new_pos, elf.to_noop())
                    } else {
                        (pos, elf.to_noop())
                    }
                }
            }
        })
        .collect()
}

fn get_corners<'a>(positions: impl Iterator<Item = &'a (i32, i32)>) -> ((i32, i32), (i32, i32)) {
    let (min_x, max_x, min_y, max_y) = positions.fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
        |(min_x, max_x, min_y, max_y), p| {
            (
                std::cmp::min(min_x, p.0),
                std::cmp::max(max_x, p.0),
                std::cmp::min(min_y, p.1),
                std::cmp::max(max_y, p.1),
            )
        },
    );
    ((min_x, min_y), (max_x, max_y))
}

fn get_empty_tiles<'a>(elves: &HashMap<(i32, i32), Elf>) -> i32 {
    let ((min_x, min_y), (max_x, max_y)) = get_corners(elves.keys());
    let mut empty_count = 0;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            empty_count += if elves.contains_key(&(x, y)) { 0 } else { 1 }
        }
    }
    empty_count
}

fn rule(
    checks: &[(i32, i32)],
    direction: (i32, i32),
) -> impl (Fn(&HashSet<(i32, i32)>, &(i32, i32)) -> Option<(i32, i32)>) + '_ {
    move |elves, old_pos| {
        checks
            .iter()
            .map(move |i| (old_pos.0 + i.0, old_pos.1 + i.1))
            .filter(|adj| elves.get(adj).is_some())
            .count()
            .eq(&0)
            .then(|| (old_pos.0 + direction.0, old_pos.1 + direction.1))
    }
}

fn simulation(input: &str, rounds: u32) -> i32 {
    let mut elves = parse(input);
    for _ in 1..=rounds {
        elves = first_half(elves);
        elves = second_half(elves);
    }
    get_empty_tiles(&elves)
}

fn simulate_until_stopped(input: &str) -> u32 {
    let mut elves = parse(input);
    let mut round = 1;
    loop {
        elves = first_half(elves);
        let active_elves = elves
            .values()
            .filter(|elf| match elf.state {
                ElfState::ProposedMove { .. } => true,
                _ => false,
            })
            .count();
        if active_elves == 0 {
            break round;
        }
        elves = second_half(elves);
        round += 1;
    }
}

// used for debugging
#[allow(dead_code)]
fn elves_to_string<'a>(elves: &HashMap<(i32, i32), Elf>) -> String {
    let ((min_x, min_y), (max_x, max_y)) = get_corners(elves.keys());
    let mut s = String::with_capacity(
        ((max_x - min_x + 1) * (max_y - min_y + 1) + (max_y - min_y + 1))
            .try_into()
            .unwrap(),
    );
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if elves.contains_key(&(x, y)) {
                s.push('#')
            } else {
                s.push(',')
            }
        }
        s.push('\n')
    }
    s
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    println!("solution 1: {}", simulation(&input, 10));
    println!("solution 2: {}", simulate_until_stopped(&input));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(simulation(include_str!("../../input/day23-test"), 10), 110);
    }

    #[test]
    fn test_example_until_stopped() {
        assert_eq!(
            simulate_until_stopped(include_str!("../../input/day23-test")),
            20
        );
    }
}
