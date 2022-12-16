use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::cmp::{max, min};
use std::collections::BTreeSet;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Clone, Debug)]
struct Valve<'a> {
    flow_rate: u32,
    tunnels: Vec<&'a str>,
}

lazy_static! {
    static ref RE_STATEMENT: Regex = RegexBuilder::new(
        r"^Valve ([A-Z][A-Z]) has flow rate=(\d+); tunnels? leads? to valves? (.+)$"
    )
    .multi_line(true)
    .build()
    .unwrap();
}

fn parse<'a>(input: &'a str) -> BTreeMap<&'a str, Valve> {
    RE_STATEMENT
        .captures_iter(input)
        .map(|m| {
            (
                m.get(1).unwrap().as_str(),
                Valve {
                    flow_rate: m.get(2).unwrap().as_str().parse::<u32>().unwrap(),
                    tunnels: m
                        .get(3)
                        .unwrap()
                        .as_str()
                        .split(", ")
                        .collect::<Vec<&str>>(),
                },
            )
        })
        .collect()
}

fn search(
    map: &BTreeMap<&str, Valve>,
    previous_positions: &BTreeSet<&str>,
    current_position: &str,
    remaining_time: u32,
) -> u32 {
    let valve = map.get(current_position).unwrap();
    let mut new_previous_positions = previous_positions.clone();
    new_previous_positions.insert(current_position);
    let mut best_flow_rate = 0;

    if remaining_time == 0 {
        return 0;
    }

    for next_position in valve.tunnels.iter() {
        let flow_without_local_release = search(
            map,
            &new_previous_positions,
            next_position,
            remaining_time - 1,
        );
        let local_flow_rate = if !previous_positions.contains(current_position) {
            valve.flow_rate
        } else {
            0
        };
        let flow_with_local_release = match remaining_time {
            t if t == 1 => local_flow_rate,
            t if t > 1 => {
                local_flow_rate + search(map, &new_previous_positions, next_position, t - 2)
            }
            _ => panic!("expected this case to be dealt with higher up"),
        };
        // NOTE: we don't need to keep track of if the previous positions has been opened.
        // This is because it can only be openned once, so we assume the first visit
        // may have opened it.
        best_flow_rate = max(
            max(flow_without_local_release, flow_with_local_release),
            best_flow_rate,
        );
    }
    best_flow_rate
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let map = parse(&input);
    let cost = search(&map, &BTreeSet::new(), "AA", 30);
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = include_str!("../../input/day16-test");
        let _ = parse(input);
    }

    #[test]
    fn test_search() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        assert_eq!(search(&map, &BTreeSet::new(), "AA", 1), 0);
        assert_eq!(search(&map, &BTreeSet::new(), "AA", 2), 20);
        assert_eq!(search(&map, &BTreeSet::new(), "AA", 3), 21);
        assert_eq!(search(&map, &BTreeSet::new(), "AA", 4), 23);
    }
}
