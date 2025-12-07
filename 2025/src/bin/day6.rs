use anyhow::Error;
use core::num;
use std::f32::consts::E;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Operation {
    Plus,
    Multiply,
}
impl Operation {
    fn init(&self) -> i64 {
        match self {
            Operation::Plus => 0,
            Operation::Multiply => 1,
        }
    }
    fn apply(&self, a: i64, b: i64) -> i64 {
        match self {
            Operation::Plus => a + b,
            Operation::Multiply => a * b,
        }
    }
}

impl TryFrom<char> for Operation {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Operation::Plus),
            '*' => Ok(Operation::Multiply),
            _ => Err(Error::msg("Invalid operation")),
        }
    }
}

fn parse(input: &str) -> (Vec<Vec<i64>>, Vec<Operation>) {
    let mut lines = input.lines().peekable();
    let mut operations = Vec::new();
    let mut numbers = Vec::new();
    while let Some(line) = lines.next() {
        if lines.peek().is_none() {
            operations = line
                .split_whitespace()
                .map(|x| match x {
                    "+" => Operation::Plus,
                    "*" => Operation::Multiply,
                    _ => panic!(""),
                })
                .collect()
        } else {
            numbers.push(
                line.split_whitespace()
                    .map(|x| x.parse().unwrap())
                    .collect(),
            );
        }
    }
    (numbers, operations)
}

fn part1(input: &str) -> i64 {
    let (numbers, operations) = parse(&input);
    let mut total = 0;
    for i in 0..numbers[0].len() {
        let op = operations[i];
        let mut val = op.init();
        for j in 0..numbers.len() {
            val = op.apply(val, numbers[j][i]);
        }
        total += val;
    }
    total
}

fn part2(input: &str) -> i64 {
    let height = input.lines().count();
    let width = input.lines().next().unwrap().len();
    let cells: Vec<Vec<char>> = input.lines().map(|line| line.chars().collect()).collect();
    let mut op: Operation = cells[height - 1][0].try_into().unwrap();
    let mut total = 0;
    let mut op_value = op.init();
    for i in 0..width {
        //op = cells[height - 1][i].try_into().unwrap_or(op);
        let m: Result<Operation, Error> = cells[height - 1][i].try_into();
        op = match m {
            Ok(x) => {
                op_value = x.init();
                x
            }
            Err(_) => op,
        };
        let mut holding = None;
        for j in 0..height - 1 {
            let c = cells[j][i];
            holding = match c.to_digit(10).map(|x| x as i64) {
                Some(d) => Some(holding.unwrap_or(0) * 10 + d),
                None => holding,
            }
        }
        match holding {
            Some(h) => op_value = op.apply(op_value, h),
            None => total += op_value,
        }
    }
    total += op_value;
    total
}

fn main() {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    println!("Part 1: {}", part1(&input));
    println!("Part 2: {}", part2(&input));
}
