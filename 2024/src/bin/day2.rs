use std::{
    cmp::Ordering,
    fs::{File, Permissions, read_to_string},
    io::BufRead,
};

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    Increasing(i32),
    Decreasing(i32),
    Unsafe,
}

impl State {
    fn next(self, &next: &i32) -> Self {
        match self {
            Self::Increasing(last) if next - last >= 1 && next - last <= 3 => {
                Self::Increasing(next)
            }
            Self::Increasing(_) => Self::Unsafe,
            Self::Decreasing(last) if last - next >= 1 && last - next <= 3 => {
                Self::Decreasing(next)
            }
            Self::Decreasing(_) => Self::Unsafe,
            Self::Unsafe => Self::Unsafe,
        }
    }
}

fn is_safe(report: &[i32]) -> bool {
    if let Some((first, report1)) = report.split_first() {
        if let Some((second, _)) = report1.split_first() {
            let initial_state = match (first - second).cmp(&0) {
                Ordering::Less => State::Increasing(*first),
                Ordering::Greater => State::Decreasing(*first),
                Ordering::Equal => State::Unsafe,
            };
            match report1.iter().fold(initial_state, State::next) {
                State::Decreasing(_) => true,
                State::Increasing(_) => true,
                State::Unsafe => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

fn permute(report: &[i32]) -> Vec<Vec<i32>> {
    let mut permutations = Vec::new();
    for i in 0..report.len() {
        let mut v = Vec::new();
        let (first, second) = report.split_at(i);
        let (_, second) = second.split_first().unwrap();
        v.extend_from_slice(first);
        v.extend_from_slice(second);
        permutations.push(v);
    }
    permutations
}

fn main() {
    let args = Cli::parse();
    let content = read_to_string(args.path).expect("could not read file");
    let reports: Result<Vec<Vec<i32>>, _> = content
        .lines()
        .map(|line| line.split(" ").map(str::parse).collect())
        .collect();
    let reports = reports.expect("Error parsing");

    let count = reports.iter().filter(|report| is_safe(report)).count();
    println!("{:?}", count);

    let count = reports
        .iter()
        .filter(|report| {
            if is_safe(report) {
                true
            } else {
                permute(report).iter().any(|x| is_safe(x))
            }
        })
        .count();
    println!("{:?}", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_case() {
        assert_eq!(State::Increasing(2).next(&7), State::Unsafe);
        assert_eq!(State::Increasing(2).next(&5), State::Increasing(5));
        assert_eq!(State::Increasing(2).next(&3), State::Increasing(3));
        assert_eq!(State::Increasing(2).next(&6), State::Unsafe);
        assert_eq!(State::Increasing(2).next(&2), State::Unsafe);
        assert_eq!(State::Increasing(2).next(&1), State::Unsafe);

        assert_eq!(State::Decreasing(2).next(&0), State::Decreasing(0));
        assert_eq!(State::Decreasing(2).next(&-1), State::Decreasing(-1));
        assert_eq!(State::Decreasing(2).next(&-2), State::Unsafe);
        assert_eq!(State::Decreasing(2).next(&2), State::Unsafe);
    }

    #[test]
    fn test_safety() {
        assert!(is_safe(&[7, 6, 4, 2, 1]));
        assert!(!is_safe(&[1, 2, 7, 8, 9]));
        assert!(!is_safe(&[9, 7, 6, 2, 1]));
        assert!(!is_safe(&[1, 3, 2, 4, 5]));
        assert!(!is_safe(&[8, 6, 4, 4, 1]));
        assert!(is_safe(&[1, 3, 6, 7, 9]));
        assert!(!is_safe(&[10, 16, 17, 20, 23]));
    }

    #[test]
    fn test_permutations() {
        assert_eq!(
            permute(&[1, 2, 3, 4]),
            vec![vec![2, 3, 4], vec![1, 3, 4], vec![1, 2, 4], vec![1, 2, 3]]
        );
    }
}
