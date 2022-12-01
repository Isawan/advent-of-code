use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

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

impl Tracker {
    fn init() -> Self {
        Tracker {
            most_carried: 0,
            latest_carried: 0,
        }
    }
}


fn chomp(state: Tracker, next: Line) -> Tracker {
    match next {
        Line::Empty => Tracker {
            most_carried: state.most_carried,
            latest_carried: 0,
        },
        Line::Number(i) => Tracker {
            most_carried: std::cmp::max(state.latest_carried + i, state.most_carried),
            latest_carried: state.latest_carried + i,
        },
    }
}

fn parse(line: &str) -> Line {
    if line == "" {
        return Line::Empty;
    }
    return Line::Number(line.parse().expect("Could not parse"));
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct RankedTracker {
    most_carried: Vec<i32>,
    latest_carried: i32,
}


fn get_top(mut heap: BinaryHeap<Reverse<i32>>, candidate: i32) -> BinaryHeap<Reverse<i32>>{
    heap.push(Reverse(candidate));
    if heap.len() >= 3 {
        let _ = heap.pop();
    }
    heap
}


fn main() {
    let args = Cli::from_args();
    println!("Error");
    let input = File::open(args.path.as_path()).unwrap();
    let lines = BufReader::new(input).lines();
    let end_state = lines.map(|x| parse(&x.unwrap())).fold(Tracker::init(), chomp);
    println!("{:?}", end_state);
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
    #[test]
    fn test_parse_panic() {}
}
