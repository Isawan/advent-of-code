use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct Network<'a> {
    edges: BTreeSet<(Cave<'a>, Cave<'a>)>,
    visit_limit: BTreeMap<Cave<'a>, u32>,
}

impl<'a> Network<'a> {
    fn new(edges: &[(Cave<'a>, Cave<'a>)]) -> Self {
        let mut network = Network {
            edges: BTreeSet::new(),
            visit_limit: BTreeMap::new(),
        };
        for edge in edges {
            network.edges.insert((edge.1, edge.0));
            network.edges.insert(*edge);
        }
        return network;
    }

    fn neighbours(&self, cave: &Cave) -> BTreeSet<&'a Cave> {
        let mut v = BTreeSet::new();
        for edge in self.edges.iter() {
            if edge.0 == *cave {
                v.insert(&edge.1);
            }
        }
        v
    }
}

#[derive(Debug)]
struct PathVisited<'a> {
    path: Vec<Cave<'a>>,
    done_second_visit: bool,
}

impl<'a> PathVisited<'a> {
    fn with_next(&self, target: Cave<'a>) -> Self {
        let mut v = Vec::new();
        v.extend_from_slice(self.as_ref());
        v.push(target);
        PathVisited {
            path: v,
            done_second_visit: self.done_second_visit,
        }
    }

    fn visited(self, visit_state: bool) -> Self {
        PathVisited {
            path: self.path,
            done_second_visit: self.done_second_visit || visit_state,
        }
    }

    fn len(&self) -> usize {
        self.path.len()
    }
    fn contains(&self, v: &'a Cave) -> bool {
        self.path.contains(v)
    }
}

impl<'a> AsRef<[Cave<'a>]> for PathVisited<'a> {
    fn as_ref(&self) -> &[Cave<'a>] {
        &self.path
    }
}

impl<'a> From<&'a [Cave<'a>]> for PathVisited<'a> {
    fn from(slice: &[Cave<'a>]) -> Self {
        PathVisited {
            path: slice.iter().map(|x| *x).collect(), 
            done_second_visit: false,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Eq, PartialOrd, Ord)]
enum Cave<'a> {
    Large(&'a str),
    Small(&'a str),
}

const START: Cave = Cave::Small("start");
const END: Cave = Cave::Small("end");

impl<'a> Cave<'a> {
    fn new(name: &'a str) -> Self {
        let is_upper = name.chars().all(|c| c.is_uppercase());
        match is_upper {
            true => Cave::Large(name),
            false => Cave::Small(name),
        }
    }
}

fn parse_network(source: &str) -> Network {
    let re = Regex::new(r"(.+?)-(.+)").unwrap();
    let mut edges = Vec::new();
    let captures = source.trim().split('\n').map(|x| re.captures(x).unwrap());
    for capture in captures {
        let k = capture.get(1).unwrap().as_str();
        let j = capture.get(2).unwrap().as_str();
        edges.push((Cave::new(k), Cave::new(j)));
    }
    Network::new(edges.as_slice())
}

fn paths_since_small_room<'a>(paths: &'a [Cave]) -> &'a [Cave<'a>] {
    for (i, cave) in paths.iter().rev().enumerate() {
        if let Cave::Small(_) = cave {
            return &paths[paths.len() - i..];
        }
    }
    return paths;
}

fn search_network_with_revisit<'a>(
    visited_paths: &PathVisited<'a>,
    network: &'a Network,
    final_target: Cave,
) -> u32 {
    assert!(visited_paths.len() > 0);
    let current_cave = &visited_paths.path[visited_paths.len() - 1];
    let mut valid_paths = 0;
    for target in network.neighbours(current_cave) {
        let mut is_revisit = false;
        // handle reaching destination
        if *target == final_target {
            valid_paths = valid_paths + 1;
            continue;
        }
        // Loop detection
        if paths_since_small_room(visited_paths.as_ref()).contains(target) {
            println!("paths_since {:?}", paths_since_small_room(visited_paths.as_ref()));
            println!("{:?}   {:?}", visited_paths, target);
            continue;
        }
        if let Cave::Small(_) = target {
            if visited_paths.contains(target) {
                if *target == START || *target == END {
                    continue;
                }
                if !visited_paths.done_second_visit {
                    is_revisit = true;
                } else {
                    continue;
                }
            }
        }
        let child_paths = search_network_with_revisit(
            &visited_paths.with_next(*target).visited(is_revisit),
            network,
            final_target,
        );
        valid_paths = valid_paths + child_paths
    }
    valid_paths
}

fn search_network<'a>(visited_paths: &[Cave<'a>], network: &'a Network, final_target: Cave) -> u32 {
    assert!(visited_paths.len() > 0);
    let current_cave = &visited_paths[visited_paths.len() - 1];
    let mut valid_paths = 0;
    for target in network.neighbours(current_cave) {
        if *target == final_target {
            valid_paths = valid_paths + 1;
            continue;
        }
        if paths_since_small_room(visited_paths).contains(target) {
            continue;
        }
        if let Cave::Small(_) = target {
            if visited_paths.contains(target) {
                continue;
            }
        }
        let mut v = Vec::new();
        v.extend_from_slice(visited_paths);
        v.push(*target);
        valid_paths = valid_paths + search_network(&v, network, final_target);
    }
    valid_paths
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let network = parse_network(&source);
    let start = Cave::new("start");
    let end = Cave::new("end");
    let valid_paths = search_network(&vec![start], &network, end);
    println!("Possible paths: {}", valid_paths);
    let valid_paths = search_network_with_revisit(&vec![start].as_slice().into(), &network, end);
    println!("Paths with revisit: {}", valid_paths);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_network() {
        let network = parse_network(
            "start-A\n\
             start-b\n\
             A-c\n\
             A-b\n\
             b-d\n\
             A-end\n\
             b-end\n",
        );
        let start = Cave::new("start");
        let c = Cave::new("c");
        let b = Cave::new("b");
        let end = Cave::new("end");
        let mut exp = BTreeSet::new();
        exp.insert(&start);
        exp.insert(&c);
        exp.insert(&b);
        exp.insert(&end);
        assert_eq!(network.neighbours(&Cave::new("A")), exp);
    }

    #[test]
    fn test_direct_adjacentcy() {
        let network = parse_network("start-end\n");
        let start = Cave::new("start");
        let end = Cave::new("end");
        let v = search_network(&vec![start], &network, end);
        assert_eq!(v, 1);
    }

    #[test]
    fn test_simple_search() {
        let network = parse_network(
            "start-A\n\
             A-B\n\
             B-end\n",
        );
        let start_node = Cave::new("start");
        let end_node = Cave::new("end");
        let start_path = vec![start_node];
        let v = search_network(&start_path, &network, end_node);
        assert_eq!(v, 1);
    }

    #[test]
    fn test_full_search() {
        let network = parse_network(
            "start-A\n\
             start-b\n\
             A-c\n\
             A-b\n\
             b-d\n\
             A-end\n\
             b-end\n",
        );
        let start_path = vec![Cave::new("start")];
        let end_node = Cave::new("end");
        let v = search_network(&start_path, &network, end_node);
        assert_eq!(v, 10);
    }

    #[test]
    fn test_panic() {
        let network = parse_network(
            "start-A\n\
             start-b\n\
             A-c\n\
             A-b\n\
             b-d\n\
             A-end\n\
             b-end\n",
        );
        let start_path = vec![Cave::new("start")];
        let end_node = Cave::new("end");
        let v = search_network_with_revisit(&start_path.as_slice().into(), &network, end_node);
        assert_eq!(v, 36);
    }
}
