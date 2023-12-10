use clap::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::{map, value};
use nom::multi::many1;
use nom::sequence::{separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::read;
use std::time::Instant;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
enum Instruction {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Node<'a> {
    name: &'a str,
    left: &'a str,
    right: &'a str,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct InstructionCycle<'a> {
    instructions: &'a [Instruction],
    pub step: usize,
}

impl<'a> InstructionCycle<'a> {
    fn new(instructions: &'a [Instruction]) -> Self {
        Self {
            instructions,
            step: 0,
        }
    }
}

impl<'a> Iterator for InstructionCycle<'a> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let instruction = self.instructions.get(self.step).unwrap();
        self.step = (self.step + 1) % self.instructions.len();
        Some(*instruction)
    }
}

fn instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
    many1(alt((
        value(Instruction::Left, nom::character::complete::char('L')),
        value(Instruction::Right, nom::character::complete::char('R')),
    )))(input)
}

fn node(input: &str) -> IResult<&str, Node> {
    map(
        tuple((
            alphanumeric1,
            tag(" = ("),
            alphanumeric1,
            tag(", "),
            alphanumeric1,
            tag(")"),
        )),
        |(name, _, left, _, right, _)| Node { name, left, right },
    )(input)
}

fn maps(input: &str) -> IResult<&str, (Vec<Instruction>, Vec<Node>)> {
    separated_pair(
        instructions,
        multispace0,
        many1(terminated(node, multispace0)),
    )(input)
}

fn traverse((instructions, nodes): &(Vec<Instruction>, Vec<Node>)) -> u64 {
    let map = nodes
        .iter()
        .map(|node| (node.name, (node.left, node.right)))
        .collect::<std::collections::HashMap<_, _>>();
    let mut current = "AAA";
    let mut steps = 0;
    let instructions = instructions.iter().cycle();
    for instruction in instructions {
        let (left, right) = map.get(current).expect("Node not found");
        current = match instruction {
            Instruction::Left => left,
            Instruction::Right => right,
        };
        steps += 1;
        if current == "ZZZ" {
            break;
        }
    }
    steps
}

fn ghost_traverse((instructions, nodes): &(Vec<Instruction>, Vec<Node>)) -> u64 {
    let map = nodes
        .iter()
        .map(|node| (node.name, (node.left, node.right)))
        .collect::<std::collections::HashMap<_, _>>();
    let mut ins_state = InstructionCycle::new(&instructions);
    let mut mem = HashMap::new();
    let mut steps = 0;
    let mut current = "AAA";
    loop {
        mem.insert((ins_state.step, current));
        let ins = ins_state.next().expect("Expected cyclic");
        println!("{:} {:?}", ins_state.step, current);
        let (left, right) = map.get(current).expect("Node not found");
        current = match ins {
            Instruction::Left => left,
            Instruction::Right => right,
        };
        steps += 1;
        if current == "ZZZ" {
            break;
        }
    }
    steps
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let input = read(args.path.as_path()).unwrap();
    let input = std::str::from_utf8(&input).unwrap();
    let (_, input) = maps(input).unwrap();
    println!("Part 1: {}", traverse(&input));
    println!("Part 2: {}", ghost_traverse(&input));

    println!("Time elapsed: {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instructions() {
        assert_eq!(
            instructions("LRLR"),
            Ok((
                "",
                vec![
                    Instruction::Left,
                    Instruction::Right,
                    Instruction::Left,
                    Instruction::Right
                ]
            ))
        );
    }

    #[test]
    fn test_node() {
        assert_eq!(
            node("AAA = (BBB, CCC)"),
            Ok((
                "",
                Node {
                    name: "AAA",
                    left: "BBB",
                    right: "CCC",
                }
            ))
        );
    }

    #[test]
    fn test_maps() {
        assert_eq!(
            maps("LRLR\nAAA = (BBB, CCC)\nBBB = (DDD, EEE)"),
            Ok((
                "",
                (
                    vec![
                        Instruction::Left,
                        Instruction::Right,
                        Instruction::Left,
                        Instruction::Right
                    ],
                    vec![
                        Node {
                            name: "AAA",
                            left: "BBB",
                            right: "CCC",
                        },
                        Node {
                            name: "BBB",
                            left: "DDD",
                            right: "EEE",
                        },
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day8-example");
        let (_, input) = maps(input).unwrap();
        assert_eq!(traverse(&input), 6);
    }

    #[test]
    fn test_equivalence() {
        let input = include_str!("../../input/day8-example");
        let (_, input) = maps(input).unwrap();
        assert_eq!(traverse(&input), ghost_traverse(&input));
    }

    //#[test]
    //fn test_ghost_example() {
    //    let input = include_str!("../../input/day8-example-ghost");
    //    let (_, input) = maps(input).unwrap();
    //    assert_eq!(ghost_traverse(&input), 6);
    //}
}
