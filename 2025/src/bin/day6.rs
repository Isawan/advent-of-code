use core::num;

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

fn part1((numbers, operations): (Vec<Vec<i64>>, Vec<Operation>)) -> i64 {
    let mut total = 0;
    for i in 0..numbers[0].len() {
        let op = operations[i];
        let mut val = op.init();
        for j in 0..numbers.len() {
            val = op.apply(val, numbers[j][i]);
        }
        println!("{val}");
        total += val;
    }
    total
}

fn main() {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    let input = parse(&input);
    println!("Part 1: {}", part1(input));
}
