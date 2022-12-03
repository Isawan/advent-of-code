use itertools::Itertools;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

type Rucksack<'a> = (Compartment<'a>, Compartment<'a>);
type Compartment<'a> = &'a str;

fn priority(c: char) -> u32 {
    let ascii_code = c as u32;
    if ascii_code >= 65 && ascii_code <= 90 {
        // A-Z
        ascii_code - 65 + 27
    } else if ascii_code >= 97 && ascii_code <= 122 {
        // a-z
        ascii_code - 97 + 1
    } else {
        panic!("Unexpected character");
    }
}

fn parse_line<'a>(line: &'a str) -> Rucksack<'a> {
    let len = line.len();
    ((&line[..len / 2]), (&line[len / 2..]))
}

fn intersect(rucksack: Rucksack) -> char {
    let mut first = BTreeSet::new();
    let mut second = BTreeSet::new();
    for i in rucksack.0.chars() {
        first.insert(i);
    }
    for i in rucksack.1.chars() {
        second.insert(i);
    }
    let mut common = first.intersection(&second);
    if let Some(single_item) = common.next() {
        return *single_item;
    }
    panic!("Unexpected");
}

fn intersect_foldable(s1: BTreeSet<char>, s2: BTreeSet<char>) -> BTreeSet<char> {
    s1.intersection(&s2).map(|x| *x).collect()
}

fn item_set(r: Rucksack) -> BTreeSet<char> {
    let mut set = BTreeSet::new();
    for i in r.0.chars() {
        set.insert(i);
    }
    for i in r.1.chars() {
        set.insert(i);
    }
    set
}

fn main() {
    let args = Cli::from_args();
    let input = File::open(args.path.as_path()).unwrap();
    let lines = BufReader::new(input).lines();
    let sum = lines
        .map(|line| intersect(parse_line(&line.unwrap())))
        .fold(0, |a, x| a + priority(x));
    println!("{:?}", sum);

    let input = File::open(args.path.as_path()).unwrap();
    let lines = BufReader::new(input).lines();
    let sum = lines
        .map(|line| item_set(parse_line(&line.unwrap())))
        .chunks(3)
        .into_iter()
        .map(|it| {
            it.reduce(intersect_foldable)
                .unwrap()
                .iter()
                .map(|x| priority(x.clone()))
                .fold(0, |a, x| a + x)
        })
        .fold(0, |a, x| a + x);
    println!("{}", sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_correct() {
        assert_eq!(priority('a'), 1);
        assert_eq!(priority('z'), 26);
        assert_eq!(priority('A'), 27);
        assert_eq!(priority('Z'), 52);
    }
    #[test]
    fn test_parse() {
        let r = parse_line("vJrwpWtwJgWrhcsFMMfFFhFp");
        assert_eq!(r.0, "vJrwpWtwJgWr");
        assert_eq!(r.1, "hcsFMMfFFhFp");
    }
    #[test]
    fn test_intersection() {
        let r = parse_line("vJrwpWtwJgWrhcsFMMfFFhFp");
        let common = intersect(r);
        assert_eq!(common, 'p');
    }
}
