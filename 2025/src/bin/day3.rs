use std::cmp::max;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn max_jolt(input: &[u8]) -> u8 {
    let mut largest = 0;
    for i in 0..(input.len() - 1) {
        for j in i + 1..input.len() {
            let n = (10 * (input[i] - b'0')) + (input[j] - b'0');
            largest = max(largest, n);
        }
    }
    largest
}

fn max_jolt2(n: usize, input: &[u8]) -> i64 {
    if n == 1 {
        return input.iter().fold(0, |a, c| max(a, (c - b'0') as i64));
    }

    let mut largest = 0;
    for i in 0..(input.len() - n + 1) {
        let head = (input[i] - b'0') as i64;
        let remain = max_jolt2(n - 1, &input[(i + 1)..]);
        let value = (10i64.pow((n as u32) - 1) * head) + remain;
        largest = max(largest, value);
    }
    return largest;
}

fn part1(input: &str) -> i32 {
    input
        .lines()
        .map(|line| max_jolt(line.as_bytes()) as i32)
        .sum()
}

fn part2(input: &str) -> i64 {
    input
        .lines()
        .map(|line| max_jolt2(12, line.as_bytes()))
        .sum()
}

fn main() {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    println!("Part 1: {}", part1(&input));
    println!("Part 2: {}", part2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_jolt() {
        assert_eq!(max_jolt("987654321111111".as_bytes()), 98);
        assert_eq!(max_jolt("811111111111119".as_bytes()), 89);
        assert_eq!(max_jolt("234234234234278".as_bytes()), 78);
        assert_eq!(max_jolt("818181911112111".as_bytes()), 92);
    }

    #[test]
    fn test_minimum_case() {
        assert_eq!(max_jolt2(1, "5678".as_bytes()), 8);
        assert_eq!(max_jolt2(1, "5".as_bytes()), 5);
    }

    #[test]
    fn test_2_case() {
        assert_eq!(max_jolt2(2, "00".as_bytes()), 0);
        assert_eq!(max_jolt2(2, "10".as_bytes()), 10);
        assert_eq!(max_jolt2(2, "123".as_bytes()), 23);
        assert_eq!(max_jolt2(2, "231".as_bytes()), 31);
        assert_eq!(max_jolt2(2, "2314".as_bytes()), 34);
    }

    #[test]
    fn test_general_case() {
        assert_eq!(max_jolt2(12, "987654321111111".as_bytes()), 987654321111);
    }
}
