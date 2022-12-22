use ndarray::ScalarOperand;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, multispace1},
    combinator::{map, map_res},
    sequence::{self, delimited, terminated, tuple},
    IResult,
};
use std::{collections::HashMap, hash::Hash, time::Instant};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation<'a> {
    Identity(i64),
    Add(&'a str, &'a str),
    Sub(&'a str, &'a str),
    Mul(&'a str, &'a str),
    Div(&'a str, &'a str),
    Eq(&'a str, &'a str),
}

impl Operation<'_> {
    fn children(&self) -> Option<(&str, &str)> {
        match self {
            Operation::Add(a, b) => Some((a, b)),
            Operation::Sub(a, b) => Some((a, b)),
            Operation::Mul(a, b) => Some((a, b)),
            Operation::Div(a, b) => Some((a, b)),
            Operation::Eq(a, b) => Some((a, b)),
            Operation::Identity(_) => None,
        }
    }
}

fn monkey<'a>(line: &'a str) -> IResult<&str, (&str, Operation<'_>)> {
    sequence::pair(
        terminated(alpha1, tag(": ")),
        alt((
            map(nom::character::complete::i64, |n| Operation::Identity(n)),
            map(
                tuple((
                    alpha1,
                    delimited(multispace1, tag("+"), multispace1),
                    alpha1,
                )),
                |(a, b, c)| Operation::Add(a, c),
            ),
            map(
                tuple((
                    alpha1,
                    delimited(multispace1, tag("-"), multispace1),
                    alpha1,
                )),
                |(a, b, c)| Operation::Sub(a, c),
            ),
            map(
                tuple((
                    alpha1,
                    delimited(multispace1, tag("*"), multispace1),
                    alpha1,
                )),
                |(a, b, c)| Operation::Mul(a, c),
            ),
            map(
                tuple((
                    alpha1,
                    delimited(multispace1, tag("/"), multispace1),
                    alpha1,
                )),
                |(a, b, c)| Operation::Div(a, c),
            ),
        )),
    )(line)
}

fn parse(input: &str) -> HashMap<&str, Operation> {
    input.lines().map(|x| monkey(x).unwrap().1).collect()
}

fn op(monkey_name: &str, context: &HashMap<&str, Operation>) -> i64 {
    let operation = context.get(monkey_name).expect("monkey not found").clone();
    let value = match operation {
        Operation::Identity(i) => i,
        Operation::Add(a, b) => op(a, context) + op(b, context),
        Operation::Sub(a, b) => op(a, context) - op(b, context),
        Operation::Mul(a, b) => op(a, context) * op(b, context),
        Operation::Div(a, b) => op(a, context) / op(b, context),
        Operation::Eq(a, b) => {
            if (op(a, context) == op(b, context)) {
                1
            } else {
                0
            }
        }
    };
    value
}

fn monkey_think(input: &str) -> i64 {
    let monkeys = parse(input);
    op("root", &monkeys)
}

fn simp_op<'a>(name: &'a str, monkeys: &mut HashMap<&'a str, Operation<'a>>) -> Option<i64> {
    if name == "humn" {
        return None;
    }
    let operation = monkeys.get(name).expect("monkey not found").clone();
    let value = match operation {
        Operation::Identity(i) => Some(i),
        Operation::Add(a, b) => simp_op(a, monkeys)
            .zip(simp_op(b, monkeys))
            .map(|(a, b)| a + b),
        Operation::Sub(a, b) => simp_op(a, monkeys)
            .zip(simp_op(b, monkeys))
            .map(|(a, b)| a - b),
        Operation::Mul(a, b) => simp_op(a, monkeys)
            .zip(simp_op(b, monkeys))
            .map(|(a, b)| a * b),
        Operation::Div(a, b) => simp_op(a, monkeys)
            .zip(simp_op(b, monkeys))
            .map(|(a, b)| a / b),
        Operation::Eq(a, b) => {
            (simp_op(a, monkeys).zip(simp_op(b, monkeys))).map(|(a, b)| if a == b { 1 } else { 0 })
        }
    };
    if let Some(v) = value {
        *monkeys.get_mut(name).unwrap() = Operation::Identity(v);
    }
    value
}

fn print_tree(name: &str, monkeys: &HashMap<&str, Operation>) {
    print!("(");
    let operation = monkeys.get(name).expect("monkey not found").clone();
    match operation {
        Operation::Identity(i) => {
            print!("{}", i)
        }
        Operation::Add(a, b) => {
            print!("+");
            print_tree(a, monkeys);
            print_tree(b, monkeys)
        }
        Operation::Sub(a, b) => {
            print!("-");
            print_tree(a, monkeys);
            print_tree(b, monkeys)
        }
        Operation::Mul(a, b) => {
            print!("*");
            print_tree(a, monkeys);
            print_tree(b, monkeys)
        }
        Operation::Div(a, b) => {
            print!("/");
            print_tree(a, monkeys);
            print_tree(b, monkeys)
        }
        Operation::Eq(a, b) => {
            print!("==");
            print_tree(a, monkeys);
            print_tree(b, monkeys)
        }
    };
    print!(")");
}

fn monkey_brute(input: &str) -> Option<i64> {
    let mut monkeys = parse(input);

    // fix root operation
    let root_monkey = monkeys.get("root").unwrap().clone();
    let (rleft, rright) = root_monkey.children().unwrap();
    monkeys.insert("root", Operation::Eq(rleft, rright));

    // fix human operation
    let root_monkey = monkeys.get("root").unwrap().clone();
    let (hleft, hright) = root_monkey.children().unwrap();
    monkeys.insert("humn", Operation::Identity(0));

    simp_op("root", &mut monkeys);
    print_tree("root", &monkeys);
    println!("reached");

    for human in 0.. {
        monkeys.insert("humn", Operation::Identity(human));
        let result = op("root", &monkeys);
        if result == 1 {
            return Some(human);
        }
    }
    None
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    println!("solution 1: {}", monkey_think(&input));
    println!("solution 2: {:?}", monkey_brute(&input));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monkey_parser() {
        assert_eq!(
            monkey("root: pppw + sjmn").unwrap().1 .1,
            Operation::Add("pppw", "sjmn")
        );
        assert_eq!(monkey("dbpl: 5").unwrap().1 .1, Operation::Identity(5));
        assert_eq!(
            monkey("ptdq: humn - dvpt").unwrap().1 .1,
            Operation::Sub("humn", "dvpt")
        );
    }

    #[test]
    fn test_monkey_think() {
        let input = include_str!("../../input/day21-test");
        assert_eq!(monkey_think(input), 152);
    }

    #[test]
    fn test_monkey_brute() {
        let input = include_str!("../../input/day21-test");
        assert_eq!(monkey_brute(input), Some(301));
    }

    #[test]
    fn test_simplify() {
        let mut monkeys = parse(include_str!("../../input/day21-test"));
        let original = op("root", &monkeys);
        simp_op("root", &mut monkeys);
        let new = op("root", &monkeys);
        assert_eq!(original, new);
    }
}
