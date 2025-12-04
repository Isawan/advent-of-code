use ahash::{HashMap, HashMapExt};
use std::cmp::max;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn max_jolt<'a>(n: usize, mem: &mut HashMap<(usize, &'a [u8]), i64>, input: &'a [u8]) -> i64 {
    if n == 1 {
        let result = input.iter().fold(0, |a, c| max(a, (c - b'0') as i64));
        mem.insert((n, input), result);
        return result;
    }

    let mut largest = 0;
    for i in 0..(input.len() - n + 1) {
        let head = (input[i] - b'0') as i64;
        let remain = match mem.get(&((n - 1), &input[(i + 1)..])) {
            Some(u) => *u,
            None => max_jolt(n - 1, mem, &input[(i + 1)..]),
        };
        let value = (10i64.pow((n as u32) - 1) * head) + remain;
        largest = max(largest, value);
    }
    mem.insert((n, input), largest);
    largest
}

fn solve(n: usize, input: &str) -> i64 {
    input
        .lines()
        .map(|line| max_jolt(n, &mut HashMap::new(), line.as_bytes()))
        .sum()
}

fn main() {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    println!("Part 1: {}", solve(2, &input));
    println!("Part 2: {}", solve(12, &input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_jolt() {
        assert_eq!(
            max_jolt(2, &mut HashMap::new(), "987654321111111".as_bytes()),
            98
        );
        assert_eq!(
            max_jolt(2, &mut HashMap::new(), "811111111111119".as_bytes()),
            89
        );
        assert_eq!(
            max_jolt(2, &mut HashMap::new(), "234234234234278".as_bytes()),
            78
        );
        assert_eq!(
            max_jolt(2, &mut HashMap::new(), "818181911112111".as_bytes()),
            92
        );
    }

    #[test]
    fn test_minimum_case() {
        assert_eq!(max_jolt(1, &mut HashMap::new(), "5678".as_bytes()), 8);
        assert_eq!(max_jolt(1, &mut HashMap::new(), "5".as_bytes()), 5);
    }

    #[test]
    fn test_2_case() {
        assert_eq!(max_jolt(2, &mut HashMap::new(), "00".as_bytes()), 0);
        assert_eq!(max_jolt(2, &mut HashMap::new(), "10".as_bytes()), 10);
        assert_eq!(max_jolt(2, &mut HashMap::new(), "123".as_bytes()), 23);
        assert_eq!(max_jolt(2, &mut HashMap::new(), "231".as_bytes()), 31);
        assert_eq!(max_jolt(2, &mut HashMap::new(), "2314".as_bytes()), 34);
    }

    #[test]
    fn test_general_case() {
        assert_eq!(
            max_jolt(12, &mut HashMap::new(), "987654321111111".as_bytes()),
            987654321111
        );
    }
}
