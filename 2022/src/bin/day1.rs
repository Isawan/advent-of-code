use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct Tracker {
    most_carried: i32,
    latest_carried: i32,
}

enum Line {
    Number(i32),
    Empty,
}
fn parse(line: &str) -> Line {
    if line == "" {
        return Line::Empty;
    }
    return Line::Number(line.parse().expect("Could not parse"));
}

#[derive(Debug, Clone)]
struct RankedTracker {
    top_carried: BinaryHeap<Reverse<i32>>,
    latest_carried: i32,
}

impl RankedTracker {
    fn init() -> Self {
        RankedTracker {
            top_carried: BinaryHeap::new(),
            latest_carried: 0,
        }
    }
}

// Pushing and popping a min heap ensures we always get k-top elements in the heap
fn get_top(topk: usize) -> impl Fn(BinaryHeap<Reverse<i32>>, i32) -> BinaryHeap<Reverse<i32>> {
    move |mut heap, candidate| {
        heap.push(Reverse(candidate));
        if heap.len() > topk {
            let _ = heap.pop();
        }
        heap
    }
}

fn ranked_chomp(state: RankedTracker, next: Line) -> RankedTracker {
    match next {
        Line::Empty => RankedTracker {
            top_carried: get_top(3)(state.top_carried, state.latest_carried),
            latest_carried: 0,
        },
        Line::Number(i) => RankedTracker {
            top_carried: state.top_carried,
            latest_carried: state.latest_carried + i,
        },
    }
}

fn main() {
    let args = Cli::from_args();
    let input = File::open(args.path.as_path()).unwrap();
    let lines = BufReader::new(input).lines();
    let ranked_state = lines
        .map(|x| parse(&x.unwrap()))
        .fold(RankedTracker::init(), ranked_chomp);
    println!("{:?}", ranked_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert_eq!(parse(""), Line::Empty);
    }
    #[test]
    fn test_parse_number() {
        assert_eq!(parse("0100"), Line::Number(100));
    }
}
