use std::{
    backtrace::{self, Backtrace},
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fs::read_to_string,
};

use anyhow::{Context, Error};
use clap::Parser;
use nom::Or;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

struct Rules(Vec<(u32, u32)>);
struct Updates(Vec<Vec<u32>>);

fn parse(input: &str) -> Result<(Rules, Updates), Error> {
    let (first, second) = input.split_once("\n\n").context("Can't split")?;
    let first = first
        .lines()
        .map(|x| {
            x.split_once('|')
                .and_then(|(x, y)| (x.parse().ok().zip(y.parse().ok())))
                .unwrap()
        })
        .collect();
    let second = second
        .lines()
        .map(|x| x.split(',').map(|x| x.parse().unwrap()).collect())
        .collect();
    Ok((Rules(first), Updates(second)))
}

#[derive(Debug, PartialEq, Eq)]
struct OrderingRules {
    after: BTreeMap<u32, BTreeSet<u32>>,
}

impl OrderingRules {
    fn is_after(&self, x: u32, y: u32) -> bool {
        //println!("{}", Backtrace::capture().to_string());
        if x == y {
            return false;
        }
        self.after.get(&x).map(|x| x.contains(&y)).unwrap_or(false)
    }
}

fn compile_rules(Rules(rules): Rules) -> OrderingRules {
    let mut after: BTreeMap<_, BTreeSet<_>> = Default::default();
    for (left, right) in rules {
        after.entry(right).or_default().insert(left);
    }
    OrderingRules { after }
}

fn part1(rules: &OrderingRules, Updates(updates): &Updates) -> u32 {
    let mut result = 0;
    'outer: for update in updates {
        let (mut last, remaining) = update.split_first().unwrap();
        let it = remaining.iter();
        for next in it {
            let r = rules.is_after(*last, *next);
            if r {
                continue 'outer;
            }
            last = next;
        }
        let d = update[update.len() / 2];
        result += d;
    }
    result
}

fn part2(rules: &OrderingRules, Updates(updates): &Updates) -> u32 {
    let mut result = 0;
    for update in updates {
        let mut update = update.clone();
        println!("-----");
        println!("{update:?}");
        let mut incorrect = false;
        loop {
            let last = update.clone();
            for i in 0..update.len() - 1 {
                let &a = update.get(i).unwrap();
                let &b = update.get(i + 1).unwrap();
                if rules.is_after(a, b) {
                    update.swap(i, i + 1);
                }
            }
            println!("{update:?}");
            if update == last {
                if incorrect {
                    let d = update[update.len() / 2];
                    result += d;
                }
                break;
            }
            incorrect = true;
        }
    }
    result
}

fn main() {
    let args = Cli::parse();
    let content = read_to_string(args.path).expect("could not read file");
    let (rules, updates) = parse(&content).unwrap();
    let rules = compile_rules(rules);
    println!("{:?}", rules);
    println!("{}", part1(&rules, &updates));
    println!("{}", part2(&rules, &updates));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moo() {
        let ordering = compile_rules(Rules(vec![(3, 4)]));
        assert!(ordering.is_after(4, 3));
        assert!(!ordering.is_after(3, 4));
        assert!(!ordering.is_after(3, 3));
        assert!(!ordering.is_after(4, 4));

        let ordering = compile_rules(Rules(vec![(3, 4), (5, 6)]));
        assert!(ordering.is_after(6, 5));

        let ordering = compile_rules(Rules(vec![(3, 4), (4, 5), (5, 6)]));
        assert!(ordering.is_after(6, 3));
    }
}
