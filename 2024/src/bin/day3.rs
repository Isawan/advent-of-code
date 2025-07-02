use clap::Parser;
use regex::{Match, Regex};
use std::fs::read_to_string;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug)]
enum Instructions {
    Mul(u32, u32),
    Do,
    Dont,
}

fn part1(content: &str) -> u32 {
    let re = Regex::new(r"mul\(([0-9]+),([0-9]+)\)").expect("Could not parse regex");
    let mut sum: u32 = 0;
    for m in re.captures_iter(content) {
        if let Some((x, y)) = m
            .get(1)
            .map(|x| x.as_str())
            .and_then(|x| x.parse::<u32>().ok())
            .zip(
                m.get(2)
                    .map(|x| x.as_str())
                    .and_then(|x| x.parse::<u32>().ok()),
            )
        {
            sum += x * y;
        }
    }
    sum
}

fn part2(content: &str) -> u32 {
    let re = Regex::new(r"(do|don't|mul)\((?:(\d+),(\d+))?\)").expect("Could not parse regex");
    let mut instructions = Vec::new();
    for m in re.captures_iter(content) {
        let ins = m.get(1).map(|x| x.as_str()).unwrap();
        instructions.push(match ins {
            "don't" => Instructions::Dont,
            "do" => Instructions::Do,
            "mul" => {
                let x = m.get(2).and_then(|x| x.as_str().parse().ok()).unwrap();
                let y = m.get(3).and_then(|x| x.as_str().parse().ok()).unwrap();
                Instructions::Mul(x, y)
            }
            x => panic!("{}", x),
        })
    }
    instructions
        .iter()
        .fold((true, 0), |(state, a), ins| match ins {
            Instructions::Do => (true, a),
            Instructions::Dont => (false, a),
            Instructions::Mul(x, y) => (state, a + if state { x * y } else { 0 }),
        })
        .1
}

fn main() {
    let args = Cli::parse();
    let content = read_to_string(args.path).expect("could not read file");
    let part1 = part1(&content);

    let part2 = part2(&content);

    println!("{}", part1);
    println!("{}", part2);
}
