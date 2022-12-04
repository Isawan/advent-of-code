use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn parse_line() -> impl Fn(&str) -> ((u32, u32), (u32, u32)) {
    let re = Regex::new(r"^(\d+)-(\d+),(\d+)-(\d+)$").unwrap();
    move |line| {
    let caps = re.captures(line).unwrap();
    (
        (
            caps.get(1).unwrap().as_str().parse::<u32>().unwrap(),
            caps.get(2).unwrap().as_str().parse::<u32>().unwrap(),
        ),
        (
            caps.get(3).unwrap().as_str().parse::<u32>().unwrap(),
            caps.get(4).unwrap().as_str().parse::<u32>().unwrap(),
        ),
    )
    }
}

fn overlap_fully(elve_pair: ((u32, u32), (u32, u32))) -> bool {
    let start = if elve_pair.0 .0 < elve_pair.1 .0 {
        elve_pair.0
    } else {
        elve_pair.1
    };
    let end = if elve_pair.0 .0 < elve_pair.1 .0 {
        elve_pair.1
    } else {
        elve_pair.0
    };
    end.1 <= start.1 || end.1 == start.1 || end.0 == start.0
}

fn overlap_at_all(elve_pair: ((u32, u32), (u32, u32))) -> bool {
    let start = if elve_pair.0 .0 < elve_pair.1 .0 {
        elve_pair.0
    } else {
        elve_pair.1
    };
    let end = if elve_pair.0 .0 < elve_pair.1 .0 {
        elve_pair.1
    } else {
        elve_pair.0
    };
    start.0 <= end.0 && start.1 >= end.0
}

fn main() {
    let args = Cli::from_args();
    let input = File::open(args.path.as_path()).unwrap();
    let lines = BufReader::new(input).lines();
    let parser = parse_line();
    let overlap_count = lines
        .map(|line| parser(&line.unwrap()))
        .filter(|x| overlap_fully(*x))
        .count();
    println!("{}", overlap_count);

    let input = File::open(args.path.as_path()).unwrap();
    let lines = BufReader::new(input).lines();
    let overlap_at_all_count = lines
        .map(|line| parser(&line.unwrap()))
        .filter(|x| overlap_at_all(*x))
        .count();
    println!("{}", overlap_at_all_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap_fully() {
        assert!(overlap_fully(((26, 85), (26, 27))));
    }

    #[test]
    fn test_overlap_at_all() {
        assert!(!overlap_at_all(((2, 4), (6, 8))));
        assert!(!overlap_at_all(((2, 3), (4, 5))));
        assert!(overlap_at_all(((5, 7), (7, 9))));
        assert!(overlap_at_all(((2, 8), (3, 7))));
        assert!(overlap_at_all(((6, 6), (4, 6))));
        assert!(overlap_at_all(((2, 6), (4, 8))));

        assert!(overlap_at_all(((26, 85), (26, 27))));
    }
}
