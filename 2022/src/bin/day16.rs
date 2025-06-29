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
    id: u32,
    destination: &'a str,
    arrival_at_remaining_time: u32,
}

impl Agent<'_> {
    fn new(id: u32, time: u32) -> Self {
        Agent {
            id,
            destination: "AA",
            arrival_at_remaining_time: time,
        }
    }
}

impl PartialOrd for Agent<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.arrival_at_remaining_time
                .cmp(&other.arrival_at_remaining_time),
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
    trace: Vec<Tracepoint<'a>>,
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

#[derive(Debug, Clone)]
struct Tracepoint<'a> {
    pressure: u32,
    total_flow_rate: u32,
    remaining_time: u32,
    to_visit: BTreeSet<&'a str>,
    agents: BinaryHeap<Agent<'a>>,
}

impl<'a> Tracepoint<'a> {
    fn new(state: &State<'a>) -> Self {
        Self {
            pressure: state.pressure,
            total_flow_rate: state.total_flow_rate,
            remaining_time: state.remaining_time,
            to_visit: state.to_visit.clone(),
            agents: state.agents.clone(),
        }
    }
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

fn search(map: &BTreeMap<&str, Valve>, agent_count: u32, time: u32) -> u32 {
    let mut queue = BinaryHeap::new();

    let mut to_visit = BTreeSet::new();
    let mut best_pressure = 0;
    let full_pressure: u32 = map.values().map(|x| x.flow_rate).sum();
    to_visit.extend(
        map.iter()
            .filter_map(|(k, v)| if v.flow_rate != 0 { Some(k) } else { None }),
    );
    let mut agents = BinaryHeap::new();
    for i in 0..agent_count {
        agents.push(Agent::new(i, time));
    }
    let mut init_state = State {
        pressure: 0,
        total_flow_rate: 0,
        remaining_time: time,
        to_visit,
        agents,
        trace: vec![],
    };
    init_state.trace.push(Tracepoint::new(&init_state));
    queue.push(init_state);
    let distance_cache = distance_cache(map);
    let mut count = 0;
    let mut best_trace = vec![];

    while let Some(state) = queue.pop() {
        count += 1;
        // out of time
        if state.remaining_time == 0 {
            let old_best = best_pressure;
            best_pressure = max(best_pressure, state.pressure);
            if old_best != best_pressure {
                println!("first");
                best_trace = state.trace.clone();
            }
            continue;
        }

        // we're about to clear the valves, calculate remaining agents then wait
        if state.to_visit.len() == 0 {
            //assert_eq!(full_pressure, state.total_flow_rate);
            // TODO: fix this bit
            let old_best = best_pressure;
            let mut total_pressure = state.pressure + state.total_flow_rate * state.remaining_time;
            let mut agents = state.agents.clone();
            let mut remaining_time = state.remaining_time;
            let mut total_flow_rate = state.total_flow_rate;
            let mut pressure = state.pressure;
            while let Some(agent) = agents.pop() {
                let next_valve = map.get(agent.destination).unwrap();
                let landing_time = state.remaining_time - agent.arrival_at_remaining_time;
                pressure = pressure + total_flow_rate * landing_time;
                total_flow_rate = total_flow_rate
            }
            best_pressure = max(best_pressure, total_pressure);
            continue;
        }

        // remove impossible cases
        // overestimate by assuming max pressure for the remaining time.
        if state.pressure + full_pressure * state.remaining_time < best_pressure {
            continue;
        }

        // leaving valve location down a tunnel
        // start by finding candidate locations
        for next_position in state.to_visit.iter() {
            let mut next_agents = state.agents.clone();
            let Agent {
                id,
                destination: curr,
                ..
            } = next_agents.pop().unwrap();

            let travel_time = *distance_cache.get(&(curr, next_position)).unwrap();

            let mut next_to_visit = state.to_visit.clone();
            next_to_visit.remove(next_position);

            // Don't travel if we're not going to get to the destination.
            // If no other agent is active, wait here until the end
            if state.remaining_time > travel_time {
                next_agents.push(Agent {
                    id,
                    destination: next_position,
                    // time travelling minus one to open the valve
                    arrival_at_remaining_time: state.remaining_time - 1 - travel_time,
                })
            }

            // Some special handling coming up, we need to figure out what happens between
            // now and the next agent to open _any_ valve.

            // grab minimum which is the next time an agent reaches the destination
            let next_updatable_agent = next_agents.peek().unwrap();
            let next_valve_updated = map.get(next_updatable_agent.destination).unwrap();

            // landing time is the minimum time until next update.
            let landing_time =
                state.remaining_time - next_updatable_agent.arrival_at_remaining_time;
            let next_pressure = state.pressure + state.total_flow_rate * landing_time;

            let mut next_state = State {
                pressure: next_pressure,
                total_flow_rate: state.total_flow_rate + next_valve_updated.flow_rate,
                remaining_time: next_updatable_agent.arrival_at_remaining_time,
                to_visit: next_to_visit,
                agents: next_agents,
                trace: state.trace.clone(),
            };
            next_state.trace.push(Tracepoint::new(&next_state));

            queue.push(next_state);
        }
    }
    println!("count: {}", count);
    println!("trace: {:?}", best_trace);
    best_pressure
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let map = parse(&input);
    //println!("solution 1: {}", search(&map, 1, args.minutes));
    println!("solution 2: {}", search(&map, 2, 26));
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
        assert_eq!(search(&map, 1, 30), 1651);
    }
}
