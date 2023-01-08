use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use std::cmp::{max, min, Reverse};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::collections::{BTreeSet, BinaryHeap};
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,

    #[structopt(long, default_value = "30")]
    minutes: u32,
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

type NonZeroPositions<'a> = BTreeSet<&'a str>;
type CurrentPosition<'a> = &'a str;
type RemainingTime = u32;
type TotalFlowRate = u32;
type Pressure = u32;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct State<'a> {
    destination: &'a str,
    arrival_at_remaining_time: u32,
}

fn distance<'a>(
    map: &BTreeMap<&'a str, Valve<'a>>,
    from: &'a str,
    to_buffer: &mut Vec<(&'a str, u32)>,
) {
    let mut heap = BinaryHeap::with_capacity(map.len());
    let mut visited = BTreeSet::new();
    heap.push((Reverse(0), from));
    while let Some((Reverse(dist), position)) = heap.pop() {
        to_buffer.push((position, dist));
        for next_position in map.get(position).unwrap().tunnels.iter() {
            if visited.contains(next_position) {
                continue;
            }
            visited.insert(next_position);
            heap.push((Reverse(dist + 1), next_position))
        }
    }
}

fn distance_cache<'a, 'b: 'a>(
    map: &'b BTreeMap<&'a str, Valve>,
) -> HashMap<(&'a str, &'a str), u32> {
    let mut to_buffer = Vec::with_capacity(map.len());
    let mut result = HashMap::new();
    for from in map.keys() {
        distance(map, from, &mut to_buffer);
        for (to, dist) in to_buffer.iter() {
            result.insert((*from, *to), *dist);
        }
    }
    result
}

fn search(map: &BTreeMap<&str, Valve>, current_position: &str, time: u32) -> Pressure {
    let mut queue: BinaryHeap<(
        Pressure,
        TotalFlowRate,
        RemainingTime,
        NonZeroPositions,
        CurrentPosition,
        Vec<&str>,
    )> = BinaryHeap::new();

    let mut nonzero_positions = BTreeSet::new();
    let mut best_pressure = 0;
    let full_pressure: u32 = map.values().map(|x| x.flow_rate).sum();
    nonzero_positions.extend(
        map.iter()
            .filter_map(|(k, v)| if v.flow_rate != 0 { Some(k) } else { None }),
    );
    queue.push((
        0,
        0,
        time,
        nonzero_positions,
        current_position,
        vec![current_position],
    ));
    let distance_cache = distance_cache(map);
    loop {
        if let Some((pressure, total_flow_rate, remaining_time, non_zeros, curr, path)) =
            queue.pop()
        {
            // out of time
            if remaining_time == 0 {
                best_pressure = max(best_pressure, pressure);
                continue;
            }

            // we've opened everything, just watch
            if non_zeros.len() == 0 {
                assert_eq!(full_pressure, total_flow_rate);
                best_pressure = max(best_pressure, pressure + total_flow_rate * remaining_time);
                continue;
            }

            // remove impossible cases
            if pressure + full_pressure * remaining_time < best_pressure {
                continue;
            }

            // leaving valve location down a tunnel
            // start by finding candidate locations
            for next_position in non_zeros.iter() {
                let next_valve = map.get(next_position).unwrap();
                let travel_time = *distance_cache.get(&(curr, next_position.clone())).unwrap();

                let mut next_path = path.clone();
                next_path.push(&next_position);

                let mut new_non_zeros = non_zeros.clone();
                new_non_zeros.remove(next_position);

                // don't travel if we're not going to get to the destination.
                // Just calculate remaining total pressure and stop this branch
                if remaining_time <= travel_time {
                    best_pressure = max(best_pressure, pressure + total_flow_rate * remaining_time);
                    continue;
                }

                let next_pressure =
                    pressure + (total_flow_rate) + ((total_flow_rate) * travel_time);

                queue.push((
                    next_pressure,
                    total_flow_rate + next_valve.flow_rate,
                    remaining_time - 1 - travel_time, // time travelling minus one to open the valve
                    new_non_zeros,
                    next_position,
                    next_path,
                ));
            }
        } else {
            break;
        }
    }
    best_pressure
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let map = parse(&input);
    println!("solution 1: {}", search(&map, "AA", args.minutes));
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
    fn test_heap_small_search() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        assert_eq!(search(&map, "AA", 1), 0);
        assert_eq!(search(&map, "AA", 2), 0);
        assert_eq!(search(&map, "AA", 3), 20);
        assert_eq!(search(&map, "AA", 4), 40);
    }

    #[test]
    fn test_distances() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        let mut to_buffer = Vec::new();
        distance(&map, "AA", &mut to_buffer);
        assert!(to_buffer.contains(&("DD", 1)));
        assert!(to_buffer.contains(&("II", 1)));
        assert!(to_buffer.contains(&("BB", 1)));
        assert!(to_buffer.contains(&("CC", 2)));
    }

    #[test]
    fn test_heap_big_search() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        assert_eq!(search(&map, "AA", 30), 1651);
    }
}
