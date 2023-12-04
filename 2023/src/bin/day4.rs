use std::{collections::HashMap, convert::TryInto, fs::read, time::Instant};

use clap::{command, Parser};
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map_res,
    error::ParseError,
    multi::many1,
    sequence::{delimited, pair, separated_pair},
    IResult,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

type Card = (u32, (Vec<u32>, Vec<u32>));

fn t<'a, 'b: 'a>(ss: &'b str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    delimited(multispace0, tag(ss), multispace0)
}

fn number(input: &str) -> IResult<&str, u32> {
    map_res(delimited(multispace0, digit1, multispace0), str::parse)(input)
}

fn card(input: &str) -> IResult<&str, Card> {
    pair(
        delimited(t("Card"), number, t(":")),
        separated_pair(many1(number), t("|"), many1(number)),
    )(input)
}

fn calc_points(card: Card) -> u32 {
    let (_, (winning_numbers, numbers)) = card;
    numbers
        .iter()
        .filter(|n| winning_numbers.contains(n))
        .fold(None, |acc, _| match acc {
            Some(v) => Some(v * 2),
            None => Some(1),
        })
        .unwrap_or(0)
        .try_into()
        .expect("cast failed")
}

fn part1(input: &str) -> u32 {
    input
        .lines()
        .map(|line| card(line).unwrap().1)
        .map(calc_points)
        .sum()
}

fn card_count(
    cards: &HashMap<u32, (Vec<u32>, Vec<u32>)>,
    mem: &mut HashMap<u32, u32>,
    id: u32,
) -> u32 {
    if let Some(count) = mem.get(&id) {
        return *count;
    }
    let child_count = cards
        .get(&id)
        .map(|(winning, numbers)| {
            numbers
                .iter()
                .filter(|v| winning.contains(v))
                .count()
                .try_into()
                .expect("cast error")
        })
        .map(|winning_count: u32| {
            (id + 1..=id + winning_count)
                .map(|i| card_count(cards, mem, i))
                .sum()
        })
        .unwrap_or(0);
    let count = 1 + child_count;
    mem.insert(id, count);
    count
}

fn part2(input: &str) -> u32 {
    let cards: HashMap<_, _> = input.lines().map(|line| card(line).unwrap().1).collect();
    let mut mem = HashMap::default();
    cards
        .keys()
        .map(|id| card_count(&cards, &mut mem, *id))
        .sum()
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let f = read(args.path.as_path()).unwrap();
    let input = std::str::from_utf8(&f).unwrap();
    println!("Part 1: {}", part1(input));
    println!("Part 2: {}", part2(input));
    println!("Time elapsed: {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card() {
        let example = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53";
        assert_eq!(
            card(example),
            Ok((
                "",
                (
                    1,
                    (vec![41, 48, 83, 86, 17], vec![83, 86, 6, 31, 17, 9, 48, 53])
                )
            ))
        );
    }

    #[test]
    fn test_calc_points() {
        assert_eq!(calc_points((1, (vec![1], vec![1, 2]))), 1);
        assert_eq!(calc_points((1, (vec![1], vec![2, 3]))), 0);
        assert_eq!(calc_points((1, (vec![1, 2], vec![1, 2]))), 2);
        assert_eq!(calc_points((1, (vec![1, 2], vec![2, 3]))), 1);
        assert_eq!(calc_points((1, (vec![1, 2], vec![3, 4]))), 0);
    }

    #[test]
    fn test_part1() {
        let example = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
                       Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
                       Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
                       Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
                       Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
                       Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";
        assert_eq!(part1(example), 13);
    }

    #[test]
    fn test_card_count() {
        let example = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
                       Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
                       Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
                       Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
                       Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
                       Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";
        let cards: HashMap<_, _> = example.lines().map(|line| card(line).unwrap().1).collect();
        let mut mem = HashMap::default();
        assert_eq!(card_count(&cards, &mut mem, 1), 15);
    }
}
