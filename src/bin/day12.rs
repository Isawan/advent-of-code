use regex::Regex;
use std::collections::BTreeSet;
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct Network<'a> {
    edges: BTreeSet<(Cave<'a>, Cave<'a>)>,
}

impl<'a> Network<'a> {
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


#[derive(PartialEq, Clone, Copy, Debug, Eq, PartialOrd, Ord)]
enum Cave<'a> {
    Large(&'a str),
    Small(&'a str),
}

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
    let mut edges = BTreeSet::new();
    let captures = source.trim().split('\n').map(|x| re.captures(x).unwrap());
    for capture in captures {
        let k = capture.get(1).unwrap().as_str();
        let j = capture.get(2).unwrap().as_str();
        edges.insert((Cave::new(k), Cave::new(j)));
        edges.insert((Cave::new(j), Cave::new(k)));
    }
    Network{ edges }
}

fn paths_since_small_room<'a>(paths: &'a [Cave]) -> &'a [Cave<'a>] {
    for (i, cave) in paths.iter().rev().enumerate() {
        if let Cave::Small(_) = cave {
            return &paths[paths.len()-i-1..]
        }
    }
    return paths
}

fn search_network<'a>(visited_paths: &[Cave<'a>], network: &'a Network, final_target: Cave) -> Vec<Vec<Cave<'a>>>{
    assert!(visited_paths.len() > 0);
    let current_cave = &visited_paths[visited_paths.len()-1];
    let mut valid_paths : Vec<Vec<Cave<'a>>> = Vec::new();
    for target in network.neighbours(current_cave) {
        if *target == final_target {
            let mut v = Vec::new();
            v.extend_from_slice(visited_paths);
            v.push(*target);
            valid_paths.push(v);
            continue;
        }
        if paths_since_small_room(visited_paths).contains(target) {
            continue;
        }
        if let Cave::Small(_) = target {
            if visited_paths.contains(target) {
                continue
            }
        }
        let mut v = Vec::new();
        v.extend_from_slice(visited_paths);
        v.push(*target);
        let mut paths = search_network(&v, network, final_target);
        valid_paths.append(&mut paths);
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
    println!("Possible paths: {}", valid_paths.len());
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
    fn test_direct_adjacentcy () {
        let network = parse_network("start-end\n");
        let start = Cave::new("start");
        let end = Cave::new("end");
        let v = search_network(&vec![start,end], &network, end);
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
        assert_eq!(v.len(), 1);
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

    }

}
