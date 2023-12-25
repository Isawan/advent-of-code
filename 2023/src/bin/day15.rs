use clap::Parser;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::alpha1,
    combinator::{map, opt, value},
    multi::many1,
    sequence::{preceded, terminated, tuple},
    IResult,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HashTable<'a> {
    table: [Vec<Slot<'a>>; 256],
}

impl Default for HashTable<'_> {
    fn default() -> Self {
        Self {
            table: std::array::from_fn(|_| Vec::default()),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Slot<'a> {
    key: &'a [u8],
    value: u32,
}

impl<'a> std::fmt::Debug for Slot<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slot")
            .field("key", &String::from_utf8_lossy(self.key))
            .field("value", &self.value)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Assign(u32),
    Remove,
}

fn hash(input: &[u8]) -> u32 {
    let mut current_value: u32 = 0;
    for s in input.iter() {
        current_value += *s as u32;
        current_value *= 17;
        current_value %= 256;
    }
    current_value
}

fn sum_of_hash(input: &str) -> u32 {
    let input: Vec<u8> = input.bytes().filter(|c| *c != b'\n').collect();
    let sequence = input.as_slice().split(|c| *c == b',');
    sequence.map(hash).sum()
}

fn instructions(input: &str) -> IResult<&str, Vec<(&[u8], Operation)>> {
    many1(terminated(
        tuple((
            map(alpha1, str::as_bytes),
            alt((
                map(
                    preceded(tag("="), nom::character::complete::u32),
                    Operation::Assign,
                ),
                value(Operation::Remove, tag("-")),
            )),
        )),
        opt(tag(",")),
    ))(input)
}

fn interpret(input: &str) -> HashTable {
    let mut table = HashTable::default();
    let instructions = instructions(input).unwrap().1;
    for (key, operation) in instructions {
        let hash = hash(key);
        match operation {
            Operation::Assign(value) => {
                match table.table[hash as usize].iter_mut().find(|s| s.key == key) {
                    Some(s) => {
                        s.value = value;
                    }
                    None => {
                        table.table[hash as usize].push(Slot { key, value });
                    }
                }
            }
            Operation::Remove => {
                table.table[hash as usize].retain(|s| s.key != key);
            }
        }
    }
    table
}

fn score(table: &HashTable) -> u32 {
    let mut score = 0;
    for (box_num, slot_num, Slot { key: _, value }) in table
        .table
        .iter()
        .enumerate()
        .flat_map(|(i, v)| v.iter().enumerate().map(move |(j, v)| (i, j, v)))
    {
        score += (box_num as u32 + 1) * (slot_num as u32 + 1) * value;
    }
    score
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    println!("Part 1: {}", sum_of_hash(&input));
    println!("Part 2: {}", score(&interpret(&input)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_of_hash() {
        assert_eq!(
            sum_of_hash("rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7"),
            1320
        );
    }

    #[test]
    fn test_parser() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        let (_, instructions) = instructions(input).unwrap();
        assert_eq!(
            instructions,
            vec![
                (b"rn".as_slice(), Operation::Assign(1)),
                (b"cm".as_slice(), Operation::Remove),
                (b"qp".as_slice(), Operation::Assign(3)),
                (b"cm".as_slice(), Operation::Assign(2)),
                (b"qp".as_slice(), Operation::Remove),
                (b"pc".as_slice(), Operation::Assign(4)),
                (b"ot".as_slice(), Operation::Assign(9)),
                (b"ab".as_slice(), Operation::Assign(5)),
                (b"pc".as_slice(), Operation::Remove),
                (b"pc".as_slice(), Operation::Assign(6)),
                (b"ot".as_slice(), Operation::Assign(7)),
            ]
        );
    }

    #[test]
    fn test_example_score() {
        let input = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
        let table = interpret(input);
        println!("{:?}", table);
        assert_eq!(score(&table), 145);
    }
}
