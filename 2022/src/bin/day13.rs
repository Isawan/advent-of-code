use lazy_static::lazy_static;
use nom::{self, IResult};
use regex::Regex;
use std::cmp::Ordering;
use std::rc::Rc;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, PartialEq, Eq, Ord)]
enum Packet {
    Number(u32),
    List(Vec<Rc<Packet>>),
}

fn packet(input: &str) -> nom::IResult<&str, Packet> {
    nom::branch::alt((
        nom::sequence::delimited(
            nom::character::complete::char('['),
            nom::combinator::map(
                nom::multi::separated_list0(nom::character::complete::char(','), |p| {
                    packet(p).map(|(a, b)| (a, Rc::new(b)))
                }),
                Packet::List,
            ),
            nom::character::complete::char(']'),
        ),
        nom::combinator::map_res(
            nom::character::complete::digit1,
            |n: &str| -> Result<Packet, _> { n.parse::<u32>().map(Packet::Number) },
        ),
    ))(input)
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(cmp(self, rhs))
    }
}

fn cmp_list(left: &Vec<Rc<Packet>>, right: &Vec<Rc<Packet>>) -> Ordering {
    let mut left_iter = left.iter();
    let mut right_iter = right.iter();
    loop {
        match (left_iter.next(), right_iter.next()) {
            (None, None) => return Ordering::Equal, // TODO: figure out matching
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(l), Some(r)) => {
                let y = cmp(l, r);
                match y {
                    Ordering::Equal => {
                        continue;
                    }
                    x => {
                        return x;
                    }
                }
            }
        }
    }
}

fn cmp(left: &Packet, right: &Packet) -> Ordering {
    match (left, right) {
        (Packet::Number(l), Packet::Number(r)) => l.cmp(&r),
        (Packet::Number(l), r @ Packet::List(_)) => {
            cmp(&Packet::List(vec![Rc::new(Packet::Number(*l))]), &r)
        }
        (l @ Packet::List(_), Packet::Number(r)) => {
            cmp(&l, &Packet::List(vec![Rc::new(Packet::Number(*r))]))
        }
        (Packet::List(l), Packet::List(r)) => cmp_list(l, r),
    }
}

fn parser(input: &str) -> (Packet, &str) {
    if let Result::Ok((i, output)) = packet(input) {
        return (output, i);
    } else {
        panic!("parse error");
    }
}

fn compare_all(input: &str) -> u32 {
    let mut lines = input.lines();
    let mut ordered_pair = vec![];
    let mut i = 0;
    loop {
        let first_line = lines.next().expect("expected first line");
        let second_line = lines.next().expect("expected second line");
        let (first, _) = parser(first_line);
        let (second, _) = parser(second_line);
        i = i + 1;

        match cmp(&first, &second) {
            Ordering::Less => ordered_pair.push(i),
            Ordering::Equal => ordered_pair.push(i),
            Ordering::Greater => {}
        }
        if let None = lines.next() {
            break;
        }
    }

    ordered_pair.iter().sum()
}

fn sort_all(input: &str) -> Vec<Packet> {
    let mut packets = input
        .lines()
        .filter(|line| line != &"")
        .map(|line| parser(line).0)
        .collect::<Vec<Packet>>();
    let mut markers = vec![parser("[[2]]").0, parser("[[6]]").0];
    packets.append(&mut markers);
    packets.sort();
    packets
}

fn find_markers(sorted_output: &Vec<Packet>) -> (Option<usize>, Option<usize>) {
    (
        sorted_output
            .iter()
            .position(|x| *x == parser("[[2]]").0)
            .map(|x| x + 1),
        sorted_output
            .iter()
            .position(|x| *x == parser("[[6]]").0)
            .map(|x| x + 1),
    )
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    println!("solution 1: {}", compare_all(&input));
    if let (Some(m1), Some(m2)) = find_markers(&sort_all(&input)) {
        println!("solution 2: {:?}", m1 * m2);
    }
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_compare_lists() {
        let left = Packet::List(vec![
            Rc::new(Packet::Number(1)),
            Rc::new(Packet::Number(1)),
            Rc::new(Packet::Number(3)),
            Rc::new(Packet::Number(1)),
            Rc::new(Packet::Number(1)),
        ]);
        let right = Packet::List(vec![
            Rc::new(Packet::Number(1)),
            Rc::new(Packet::Number(1)),
            Rc::new(Packet::Number(5)),
            Rc::new(Packet::Number(1)),
            Rc::new(Packet::Number(1)),
        ]);
        assert_eq!(cmp(&left, &right), Ordering::Less);
    }

    #[test]
    fn test_compare_example() {
        assert_eq!(
            cmp(&parser("[[1],[2,3,4]]").0, &parser("[[1],4]").0),
            Ordering::Less
        );
        assert_eq!(
            cmp(&parser("[[1],[]]").0, &parser("[[1],4]").0),
            Ordering::Less
        );
        assert_eq!(
            cmp(&parser("[[10],[]]").0, &parser("[[11],4]").0),
            Ordering::Less
        );
    }

    #[test]
    fn test_parser() {
        assert_eq!(parser("1000"), (Packet::Number(1000), ""));
        assert_eq!(parser("[]"), (Packet::List(vec![]), ""));
        assert_eq!(
            parser("[1000]"),
            (Packet::List(vec![Rc::new(Packet::Number(1000))]), "")
        );
        assert_eq!(
            parser("[[1],[2,3,4]]"),
            (
                Packet::List(vec![
                    Rc::new(Packet::List(vec![Rc::new(Packet::Number(1))])),
                    Rc::new(Packet::List(vec![
                        Rc::new(Packet::Number(2)),
                        Rc::new(Packet::Number(3)),
                        Rc::new(Packet::Number(4)),
                    ]))
                ]),
                ""
            )
        );
        assert_eq!(
            parser("[[1],[[],[]]]"),
            (
                Packet::List(vec![
                    Rc::new(Packet::List(vec![Rc::new(Packet::Number(1))])),
                    Rc::new(Packet::List(vec![
                        Rc::new(Packet::List(vec![])),
                        Rc::new(Packet::List(vec![]))
                    ]))
                ]),
                ""
            )
        );
    }

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day13-test");
        assert_eq!(compare_all(input), 13);
    }

    #[test]
    fn test_sort() {
        let input = include_str!("../../input/day13-test");
        let exp_output_string = include_str!("../../input/day13-test-output");
        let exp_output = exp_output_string
            .lines()
            .map(|line| parser(line).0)
            .collect::<Vec<Packet>>();
        let output = sort_all(input);
        assert_eq!(output, exp_output);
    }

    #[test]
    fn test_find_markers() {
        let input = include_str!("../../input/day13-test");
        let output = sort_all(input);
        assert_eq!(find_markers(&output), (Some(10), Some(14)));
    }
}
