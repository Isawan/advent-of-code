use std::{
    fs::{self, read_to_string},
    path::Path,
};

use clap::Parser;
use itertools::Itertools;
use ndarray::iter;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::newline,
    combinator::{map, map_res, opt},
    multi::{many1, separated_list1},
    sequence::tuple,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, Eq, PartialEq)]
struct Equation {
    result: i64,
    test_values: Vec<i64>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Operator {
    Mul,
    Add,
    Cat,
}

fn concat(a: i64, b: i64) -> i64 {
    let mut p = 10;
    while p <= b {
        p *= 10;
    }
    (a * p) + b
}

impl Operator {
    fn apply(&self, a: i64, b: i64) -> i64 {
        match self {
            Operator::Add => a + b,
            Operator::Mul => a * b,
            Operator::Cat => concat(a, b),
        }
    }
}

fn parse_line(input: &str) -> IResult<&str, Equation> {
    map(
        tuple((
            nom::character::complete::i64,
            tag(": "),
            separated_list1(tag(" "), nom::character::complete::i64),
        )),
        |(result, _, test_values)| Equation {
            result,
            test_values,
        },
    )(input)
}

fn parse(input: &str) -> IResult<&str, Vec<Equation>> {
    separated_list1(newline, parse_line)(input)
}

fn is_calibrated(equation: &Equation, operators: &[Operator]) -> bool {
    let operators_combinations = (0..equation.test_values.len() - 1)
        .map(|_| operators)
        .multi_cartesian_product();
    for operators in operators_combinations {
        let mut remaining_values = &equation.test_values[..];
        let top;
        (top, remaining_values) = remaining_values.split_first().expect("Empty equation");
        let mut remaining_operators = &operators[..];

        let mut result = *top;
        loop {
            match (
                remaining_values.split_first(),
                remaining_operators.split_first(),
            ) {
                (Some((&next_value, rem)), Some((op, op_rem))) => {
                    result = op.apply(result, next_value);
                    remaining_values = rem;
                    remaining_operators = op_rem;
                }
                (None, None) => {
                    if result == equation.result {
                        return true;
                    }
                    break;
                }
                (x, y) => {
                    panic!("Unexpected {x:?} {y:?} {result:?}");
                }
            }
        }
    }
    false
}

fn solve<'a>(equations: impl Iterator<Item = &'a Equation>, operators: &[Operator]) -> i64 {
    equations
        .filter(|eq| is_calibrated(eq, operators))
        .map(|eq| eq.result)
        .sum()
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let content = read_to_string(args.path)?;
    let (remaining, equations) = parse(&content).unwrap();
    let p = Path::new("testfile");
    assert!(remaining.is_empty());
    println!(
        "part 1:  {}",
        solve(equations.iter(), &[Operator::Add, Operator::Mul]),
    );
    println!(
        "part 2:  {}",
        solve(
            equations.iter(),
            &[Operator::Add, Operator::Mul, Operator::Cat]
        ),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        assert_eq!(
            parse_line("190: 10 19").unwrap(),
            (
                "",
                Equation {
                    result: 190,
                    test_values: vec![10, 19]
                }
            )
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(Operator::Add.apply(10, 40), 50);
        assert_eq!(Operator::Mul.apply(10, 40), 400);
    }

    #[test]
    fn test_split() {
        let (&first, remaining_values) = [1, 2, 3, 4, 5].split_first().expect("Empty equation");
        assert_eq!(first, 1);
        assert_eq!(remaining_values, [2, 3, 4, 5]);
    }

    #[test]
    fn test_weird() {
        assert!(!is_calibrated(
            &Equation {
                result: 133,
                test_values: vec![4, 1, 2, 7, 63, 30]
            },
            &[Operator::Add, Operator::Mul]
        ),);
    }

    #[test]
    fn test_concat() {
        assert_eq!(concat(10, 20), 1020);
        assert_eq!(concat(11, 20), 1120);
        assert_eq!(concat(110, 20), 11020);
        assert_eq!(concat(110, 2025), 1102025);
        assert_eq!(concat(110, 10), 11010);
    }

    #[test]
    fn test_calibration() {
        assert!(is_calibrated(
            &Equation {
                result: 190,
                test_values: vec![10, 19],
            },
            &[Operator::Add, Operator::Mul]
        ));
        assert!(is_calibrated(
            &Equation {
                result: 3267,
                test_values: vec![81, 40, 27],
            },
            &[Operator::Add, Operator::Mul]
        ));
        assert!(is_calibrated(
            &Equation {
                result: 292,
                test_values: vec![11, 6, 16, 20],
            },
            &[Operator::Add, Operator::Mul]
        ));
        assert!(!is_calibrated(
            &Equation {
                result: 83,
                test_values: vec![17, 5],
            },
            &[Operator::Add, Operator::Mul]
        ))
    }
}
