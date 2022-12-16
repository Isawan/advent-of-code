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

type PreviousPositions<'a> = BTreeSet<&'a str>;
type NonZeroPositions<'a> = BTreeSet<&'a str>;
type CurrentPosition<'a> = &'a str;
type RemainingTime = u32;
type TotalFlowRate = u32;
type Pressure = u32;

/// The best possible case is to encounter the top sorted flow rates
/// in a row, taking 2 turns each time. This means the sum of the
/// top (remaining_time/2) elements plus the full pressure of all the
/// valves after that
fn maximum_obtainable_pressure<'a>(
    map: &BTreeMap<&str, Valve>,
    visited: &BTreeSet<&str>,
    remaining_time: u32,
    full_pressure: u32,
) -> u32 {
    let mut remaining_flows: Vec<u32> = map
        .iter()
        .filter(|(k, v)| !visited.contains(*k))
        .map(|(k, v)| v.flow_rate)
        .collect();
    remaining_flows.sort_by(|a, b| b.cmp(&a));
    let minimum_minutes = ((remaining_time + 1) / 2);
    let mut building_pressure = 0;
    let mut tmp = 0;
    for r in remaining_flows.iter().take(minimum_minutes as usize) {
        tmp = tmp + r;
        building_pressure = building_pressure + tmp;
    }
    full_pressure * (remaining_time - minimum_minutes) + building_pressure
}

//fn heap_search(map: &BTreeMap<&str, Valve>, current_position: &str, time: u32) -> Pressure {
//    let mut best_pressure = 0;
//    let mut heap: BinaryHeap<(
//        Pressure,
//        TotalFlowRate,
//        Reverse<RemainingTime>,
//        PreviousPositions,
//        CurrentPosition,
//        Vec<&str>,
//    )> = BinaryHeap::new();
//    let full_pressure: u32 = map.values().map(|x| x.flow_rate).sum();
//    heap.push((
//        0,
//        0,
//        Reverse(time),
//        BTreeSet::new(),
//        current_position,
//        vec![current_position],
//    ));
//    loop {
//        if let Some((pressure, total_flow_rate, Reverse(remaining_time), prev, curr, path)) =
//            heap.pop()
//        {
//            let valve = map.get(curr).unwrap();
//            let mut new_prev = prev.clone();
//            new_prev.insert(curr);
//            //println!(
//            //    "path:{:?}, pressure: {:?} , totol_flow_rate:{:?}, remaining: {:?}, prev: {:?}, cur: {:?}",
//            //    path, pressure, total_flow_rate, remaining_time, prev, curr
//            //);
//
//            // let remaining_pressure = maximum_obtainable_pressure(map, &prev, remaining_time, full_pressure);
//            //
//            // skip if impossible to get best
//            //if remaining_pressure < best_pressure {
//            //    continue;
//            //}
//
//            // out of time
//            if remaining_time == 0 {
//                best_pressure = max(best_pressure, pressure);
//                continue;
//            }
//
//            // if all valves have been openned, its straightforward to determine total pressure
//            // without simulation
//            if map.keys().filter(|x| !prev.contains(*x)).count() == 0 {
//                best_pressure = max(best_pressure, pressure + full_pressure * remaining_time)
//            }
//
//            for next_position in valve.tunnels.iter() {
//                let mut next_path = path.clone();
//                next_path.push(&next_position);
//
//                // search local_flow without release
//                heap.push((
//                    pressure + total_flow_rate,
//                    total_flow_rate,
//                    Reverse(remaining_time - 1),
//                    new_prev.clone(),
//                    next_position,
//                    next_path.clone(),
//                ));
//
//                let local_flow_rate = if !prev.contains(curr) {
//                    valve.flow_rate
//                } else {
//                    0
//                };
//                let next_flow_rate = total_flow_rate + local_flow_rate;
//
//                if remaining_time == 1 {
//                    best_pressure = max(best_pressure, pressure + next_flow_rate);
//                } else if remaining_time > 1 {
//                    heap.push((
//                        pressure + total_flow_rate + next_flow_rate,
//                        next_flow_rate,
//                        Reverse(remaining_time - 2),
//                        new_prev.clone(),
//                        next_position,
//                        next_path.clone(),
//                    ));
//                } else {
//                    panic!("impossible condition");
//                }
//            }
//        } else {
//            break;
//        }
//    }
//    best_pressure
//}

fn distance(map: &BTreeMap<&str, Valve>, from: &str, to: &str) -> Option<u32> {
    let mut heap = BinaryHeap::new();
    let mut visited = BTreeSet::new();
    heap.push((0, from));
    loop {
        if let (dist, position) = heap.pop()? {
            if position == to {
                return Some(dist);
            }
            for next_position in map.get(position).unwrap().tunnels.iter() {
                if visited.contains(next_position) {
                    continue;
                }
                visited.insert(next_position);
                heap.push((dist + 1, next_position))
            }
        }
    }
}

fn search(map: &BTreeMap<&str, Valve>, current_position: &str, time: u32) -> Pressure {
    let mut queue: VecDeque<(
        Pressure,
        TotalFlowRate,
        RemainingTime,
        NonZeroPositions,
        CurrentPosition,
        Vec<&str>,
    )> = VecDeque::new();

    let mut nonzero_positions = BTreeSet::new();
    let mut best_pressure = 0;
    let full_pressure: u32 = map.values().map(|x| x.flow_rate).sum();
    nonzero_positions.extend(
        map.iter()
            .filter_map(|(k, v)| if v.flow_rate != 0 { Some(k) } else { None }),
    );
    queue.push_back((0, 0, time, nonzero_positions, current_position, vec![]));
    loop {
        if let Some((pressure, total_flow_rate, remaining_time, non_zeros, curr, path)) =
            queue.pop_front()
        {
            // out of time
            if remaining_time == 0 {
                best_pressure = max(best_pressure, pressure);
                continue;
            }

            // we've openned everything, just watch
            if non_zeros.len() == 0 {
                best_pressure = max(
                    best_pressure,
                    pressure + total_flow_rate * (remaining_time - 1),
                );
                continue;
            }

            // // spend one minute openning then call self with zero time to clean up
            // if remaining_time == 1 {
            //     let mut new_non_zeros = non_zeros.clone();
            //     let c = new_non_zeros.remove(curr);
            //     queue.push_back((
            //         pressure + total_flow_rate + valve.flow_rate,
            //         total_flow_rate + valve.flow_rate,
            //         0,
            //         new_non_zeros,
            //         curr,
            //         path.clone(),
            //     ));
            //     continue;
            // }

            // leaving valve location down a tunnel
            // start by finding candidate locations
            for next_position in non_zeros.iter() {
                let next_valve = map.get(next_position).unwrap();
                let travel_time = distance(map, curr, next_position).unwrap();

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

                queue.push_back((
                    pressure
                        + (total_flow_rate + next_valve.flow_rate)
                        + ((total_flow_rate) * travel_time),
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
    println!("{}", map.values().map(|x| x.flow_rate).sum::<u32>());
    println!("solution 1: {}", search(&map, "AA", 30));
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
        assert_eq!(search(&map, "AA", 2), 20);
        assert_eq!(search(&map, "AA", 3), 40);
    }

    #[test]
    fn test_distances() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        assert_eq!(distance(&map, "AA", "DD"), Some(1));
        assert_eq!(distance(&map, "AA", "II"), Some(1));
        assert_eq!(distance(&map, "AA", "BB"), Some(1));
        assert_eq!(distance(&map, "AA", "CC"), Some(2));
    }

    #[test]
    fn test_heap_big_search() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        assert_eq!(search(&map, "AA", 30), 1651);
    }
}
