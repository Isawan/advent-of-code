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

#[derive(Debug, Clone, PartialEq, Eq, Ord)]
struct Agent<'a> {
    destination: &'a str,
    arrival_at_remaining_time: u32,
}

impl Agent<'_> {
    fn new() -> Self {
        Agent {
            destination: "AA",
            arrival_at_remaining_time: 0,
        }
    }
}

impl PartialOrd for Agent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.arrival_at_remaining_time
                .cmp(&self.arrival_at_remaining_time),
        )
    }
}

#[derive(Debug, Clone)]
struct State<'a> {
    pressure: u32,
    total_flow_rate: u32,
    remaining_time: u32,
    to_visit: BTreeSet<&'a str>,
    agents: BinaryHeap<Agent<'a>>,
}

impl PartialOrd for State<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.pressure.cmp(&self.pressure))
    }
}

impl Ord for State<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for State<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.pressure == other.pressure
    }
}

impl Eq for State<'_> {}

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

fn search(map: &BTreeMap<&str, Valve>, agents: u32, time: u32) -> u32 {
    let mut queue = BinaryHeap::new();

    let mut to_visit = BTreeSet::new();
    let mut best_pressure = 0;
    let full_pressure: u32 = map.values().map(|x| x.flow_rate).sum();
    to_visit.extend(
        map.iter()
            .filter_map(|(k, v)| if v.flow_rate != 0 { Some(k) } else { None }),
    );
    let mut agents = BinaryHeap::new();
    agents.push(Agent::new());

    queue.push(State {
        pressure: 0,
        total_flow_rate: 0,
        remaining_time: time,
        to_visit,
        agents,
    });
    let distance_cache = distance_cache(map);

    while let Some(state) = queue.pop() {
        let State {
            pressure,
            total_flow_rate,
            remaining_time,
            to_visit,
            agents,
        } = state;
        // out of time
        if remaining_time == 0 {
            best_pressure = max(best_pressure, pressure);
            continue;
        }

        // we've opened everything, just watch
        if to_visit.len() == 0 {
            assert_eq!(full_pressure, total_flow_rate);
            best_pressure = max(best_pressure, pressure + total_flow_rate * remaining_time);
            continue;
        }

        // remove impossible cases
        // overestimate by assuming max pressure for the remaining time.
        if pressure + full_pressure * remaining_time < best_pressure {
            continue;
        }

        // leaving valve location down a tunnel
        // start by finding candidate locations
        for next_position in to_visit.iter() {
            let mut next_agents = agents.clone();
            let Agent {
                destination: curr, ..
            } = next_agents.pop().unwrap();

            let next_valve = map.get(next_position).unwrap();
            let travel_time = *distance_cache.get(&(curr, next_position.clone())).unwrap();

            let mut new_to_visit = to_visit.clone();
            new_to_visit.remove(next_position);

            // Don't travel if we're not going to get to the destination.
            // If no other agent is active,
            // just calculate remaining total pressure and remove agent from active set
            if next_agents.len() == 0 && remaining_time <= travel_time {
                best_pressure = max(best_pressure, pressure + total_flow_rate * remaining_time);
                continue;
            } else {
                next_agents.push(Agent {
                    destination: next_position,
                    arrival_at_remaining_time: remaining_time - 1 - travel_time, // time travelling minus one to open the valve
                })
            }

            let next_pressure = pressure + (total_flow_rate) + ((total_flow_rate) * travel_time);

            // grab minimum which is the next time an agent reaches the destination
            let remaining_time = next_agents
                .iter()
                .fold(u32::MAX, |x, agent| min(x, agent.arrival_at_remaining_time));

            queue.push(State {
                pressure: next_pressure,
                total_flow_rate: total_flow_rate + next_valve.flow_rate,
                remaining_time,
                to_visit: new_to_visit,
                agents: next_agents,
            });
        }
    }
    best_pressure
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let map = parse(&input);
    println!("solution 1: {}", search(&map, 1, args.minutes));
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
        assert_eq!(search(&map, 1, 1), 0);
        assert_eq!(search(&map, 1, 2), 0);
        assert_eq!(search(&map, 1, 3), 20);
        assert_eq!(search(&map, 1, 4), 40);
    }

    #[test]
    fn test_distances() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        let mut to_buffer = Vec::new();
        distance(&map, 1, &mut to_buffer);
        assert!(to_buffer.contains(&("DD", 1)));
        assert!(to_buffer.contains(&("II", 1)));
        assert!(to_buffer.contains(&("BB", 1)));
        assert!(to_buffer.contains(&("CC", 2)));
    }

    #[test]
    fn test_heap_big_search() {
        let input = include_str!("../../input/day16-test");
        let map = parse(input);
        assert_eq!(search(&map, 1, 30), 1651);
    }
}
