use std::fs::read_to_string;

use ahash::{HashMap, HashSet};
use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

struct Rules(Vec<(u32, u32)>);
struct Updates(Vec<Vec<u32>>);

fn parse(input: &str) -> Option<(Rules, Updates)> {
    let (first, second) = input.split_once("\n\n")?;
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
    Some((Rules(first), Updates(second)))
}

#[derive(Debug)]
struct PartialOrder {
    after: HashMap<u32, HashSet<u32>>,
}

impl PartialOrder {
    fn is_after(&self, x: u32, y: u32) -> bool {
        if x == y {
            return false;
        }
        if let Some(numbers) = self.after.get(&x) {
            for &number in numbers {
                if number == y {
                    return true;
                }
                if self.is_after(number, y) {
                    return true;
                }
            }
        }
        false
    }
}

fn compile_rules(Rules(rules): Rules) -> PartialOrder {
    let mut after: HashMap<_, HashSet<_>> = HashMap::default();
    for (left, right) in rules {
        after.entry(right).or_default().insert(left);
    }
    PartialOrder { after }
}

fn main() {
    let args = Cli::parse();
    let content = read_to_string(args.path).expect("could not read file");
    let (rules, updates) = parse(&content).unwrap();
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
