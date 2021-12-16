use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::iter::FromIterator;
use std::array::IntoIter;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Rule {
    doublet: [char; 2],
    out: char,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Polymer {
    doublets: BTreeMap<[char; 2], u64>,
    elements: BTreeMap<char, u64>,
}

impl Polymer {
    fn new(state: &str) -> Self {
        let mut doublets = BTreeMap::new();
        let mut elements = BTreeMap::new();
        let mut stream = state.chars();
        let mut last = stream.next().unwrap();
        loop {
            if let Some(next) = stream.next() {
                let count = doublets.entry([last, next]).or_insert(0);
                *count = *count + 1;
                last = next
            } else {
                break;
            }
        }
        for c in state.chars() {
            let count = elements.entry(c).or_insert(0);
            *count = *count + 1;
        }

        Polymer { doublets, elements }
    }

    fn most_common(&self) -> (char, u64) {
        self.elements
            .iter()
            .fold((' ', 0), |a, x| if *x.1 >= a.1 { (*x.0, *x.1) } else { a })
    }

    fn least_common(&self) -> (char, u64) {
        self.elements.iter().fold(
            (' ', u64::MAX),
            |a, x| {
                if *x.1 <= a.1 {
                    (*x.0, *x.1)
                } else {
                    a
                }
            },
        )
    }
}

fn parse_rule(line: &str) -> Rule {
    let re = Regex::new(r"([A-Z])([A-Z]) -> ([A-Z])").unwrap();
    let cap = re.captures(line).unwrap();
    Rule {
        doublet: [
            cap.get(1).unwrap().as_str().chars().next().unwrap(),
            cap.get(2).unwrap().as_str().chars().next().unwrap(),
        ],
        out: cap.get(3).unwrap().as_str().chars().next().unwrap(),
    }
}

fn parse_input(source: &str) -> (&str, Vec<Rule>) {
    let i = source.find('\n').unwrap();
    let (start, remain) = source.split_at(i);
    let mut r = Vec::new();
    for line in remain[2..].trim().split('\n') {
        r.push(parse_rule(line));
    }
    (start, r)
}

fn apply_rules(rules: &[Rule], state: &mut Polymer) {
    let mut new_doublets = BTreeMap::new();
    let mut new_elements = BTreeMap::new();
    for rule in rules {
        if let Some(old_count) = state.doublets.remove(&rule.doublet) {
            // deal with dublets
            let x = [rule.doublet[0], rule.out];
            let v = new_doublets.entry(x).or_insert(0);
            *v = *v + old_count;

            let x = [rule.out, rule.doublet[1]];
            let v = new_doublets.entry(x).or_insert(0);
            *v = *v + old_count;

            // deal with new elements
            let x = rule.out;
            let v = new_elements.entry(x).or_insert(0);
            *v = *v + old_count;
        }
    }
    // add in the new doublets
    for (doublet, count) in new_doublets {
        let v = state.doublets.entry(doublet).or_insert(0);
        *v = *v + count;
    }
    // add in the new elements
    for (element, count) in new_elements {
        let v = state.elements.entry(element).or_insert(0);
        *v = *v + count;
    }
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let (s, rules) = parse_input(&source);
    let mut state = Polymer::new(s);
    for _ in 0..40 {
        apply_rules(&rules, &mut state);
    }
    let (least_char, least_count) = state.least_common();
    let (most_char, most_count) = state.most_common();
    println!("most common:  {} {}", least_char, most_count);
    println!("lease common: {} {}", most_char, least_count);
    println!("diff between least and most: {}", most_count - least_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule() {
        let input = "NNCB\n\
                     \n\
                     CH -> B\n\
                     HH -> N\n\
                     CB -> H\n\
                     NH -> C\n";
        let (s, rules) = parse_input(input);
        assert_eq!(s, "NNCB");
        assert_eq!(
            rules[0],
            Rule {
                doublet: ['C', 'H'],
                out: 'B'
            }
        );
        assert_eq!(
            rules[1],
            Rule {
                doublet: ['H', 'H'],
                out: 'N'
            }
        );
        assert_eq!(
            rules[2],
            Rule {
                doublet: ['C', 'B'],
                out: 'H'
            }
        );
        assert_eq!(
            rules[3],
            Rule {
                doublet: ['N', 'H'],
                out: 'C'
            }
        );
    }

    #[test]
    fn test_build_counter() {
        let input = "NNCB";
        let state = Polymer::new(input);
        let doublet_exp = BTreeMap::from_iter(IntoIter::new([
            (['N', 'N'], 1),
            (['N', 'C'], 1),
            (['C', 'B'], 1),
        ]));
        let element_exp = BTreeMap::from_iter(IntoIter::new([('N', 2), ('C', 1), ('B', 1)]));
        assert_eq!(state.doublets, doublet_exp);
        assert_eq!(state.elements, element_exp);
    }

    #[test]
    fn test_apply_single_rule() {
        let input = "NNCB";
        let mut state = Polymer::new(input);
        let rule = Rule {
            doublet: ['N', 'C'],
            out: 'B',
        };
        apply_rules(&[rule], &mut state);
        let doublet_exp = BTreeMap::from_iter(IntoIter::new([
            (['N', 'N'], 1),
            (['N', 'B'], 1),
            (['B', 'C'], 1),
            (['C', 'B'], 1),
        ]));
        let element_exp = BTreeMap::from_iter(IntoIter::new([('N', 2), ('B', 2), ('C', 1)]));
        assert_eq!(state.doublets, doublet_exp);
        assert_eq!(state.elements, element_exp);
    }

    #[test]
    fn test_integration() {
        let input = "NNCB\n\
                     \n\
                     CH -> B\n\
                     HH -> N\n\
                     CB -> H\n\
                     NH -> C\n\
                     HB -> C\n\
                     HC -> B\n\
                     HN -> C\n\
                     NN -> C\n\
                     BH -> H\n\
                     NC -> B\n\
                     NB -> B\n\
                     BN -> B\n\
                     BB -> N\n\
                     BC -> B\n\
                     CC -> N\n\
                     CN -> C\n";
        let (s, rules) = parse_input(input);
        let mut state = Polymer::new(s);
        for _ in 0..10 {
            apply_rules(&rules, &mut state);
        }
        assert_eq!(state.elements.values().fold(0, |a, x| a + x), 3073);
    }
}
