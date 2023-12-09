use std::{
    cmp::{max, Ordering},
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    fs::read,
    time::Instant,
};

use clap::Parser;
use ndarray::Order;
use nom::{
    character::complete::{anychar, multispace1, newline},
    combinator::{map, map_opt, map_parser, map_res},
    multi::{many0, many1},
    sequence::{separated_pair, terminated, tuple},
    IResult, InputLength, Parser as NomParser,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn card(input: &str) -> IResult<&str, Card> {
    map_opt(anychar, |c| match c {
        'T' => Some(Card::T),
        'J' => Some(Card::J),
        'Q' => Some(Card::Q),
        'K' => Some(Card::K),
        'A' => Some(Card::A),
        _ => c.to_digit(10).map(Card::Number),
    })(input)
}

fn bid(input: &str) -> IResult<&str, u32> {
    nom::character::complete::u32(input)
}

fn hand(input: &str) -> IResult<&str, Hand> {
    map_res(tuple((card, card, card, card, card)), |f| f.try_into())(input)
}

fn camel_card(input: &str) -> IResult<&str, Vec<(Hand, u32)>> {
    many1(terminated(
        separated_pair(hand, multispace1, bid),
        many0(multispace1),
    ))(input)
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Copy, Clone)]
enum Card {
    Number(u32),
    T,
    J,
    Q,
    K,
    A,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Copy, Clone)]
enum JokerCard {
    J,
    Number(u32),
    T,
    Q,
    K,
    A,
}

impl From<Card> for JokerCard {
    fn from(card: Card) -> Self {
        match card {
            Card::Number(n) => JokerCard::Number(n),
            Card::T => JokerCard::T,
            Card::J => JokerCard::J,
            Card::Q => JokerCard::Q,
            Card::K => JokerCard::K,
            Card::A => JokerCard::A,
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone, Debug)]
enum HandType {
    None,
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

type Hand = [Card; 5];

fn hand_type(hand: &Hand) -> HandType {
    let mut counts: BTreeMap<&Card, u32> = BTreeMap::new();
    for card in hand.iter() {
        *counts.entry(card).or_default() += 1;
    }
    if counts.values().any(|&v| v == 5) {
        HandType::FiveOfAKind
    } else if counts.values().any(|&v| v == 4) {
        HandType::FourOfAKind
    } else if counts.values().any(|&v| v == 3) && counts.values().any(|&v| v == 2) {
        HandType::FullHouse
    } else if counts.values().any(|&v| v == 3) {
        HandType::ThreeOfAKind
    } else if counts.values().filter(|&&v| v == 2).count() == 2 {
        HandType::TwoPair
    } else if counts.values().filter(|&&v| v == 2).count() == 1 {
        HandType::OnePair
    } else if counts.values().filter(|&&v| v == 1).count() == 5 {
        HandType::HighCard
    } else {
        HandType::None
    }
}

fn total_winnings(mut hands: Vec<(Hand, u32)>) -> u32 {
    hands.sort_by(|(a, _), (b, _)| (hand_type(a), a).cmp(&(hand_type(b), b)));
    hands
        .iter()
        .enumerate()
        .fold(0, |acc, (i, (_, bid))| acc + (i as u32 + 1) * bid)
}

fn joker_type(hand: &Hand) -> HandType {
    let mut best_type = hand_type(hand);
    let mut counts: BTreeMap<&Card, u32> = BTreeMap::new();
    for card in hand.iter() {
        *counts.entry(card).or_default() += 1;
    }
    // iterate over all combination of J substitutes
    for try_card in counts.keys() {
        let derived_hand: Hand = hand
            .iter()
            .map(|card| if *card == Card::J { *try_card } else { &card })
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        best_type = max(hand_type(&derived_hand), best_type)
    }
    best_type
}

fn joker_total_winnings(mut hands: Vec<(Hand, u32)>) -> u32 {
    hands.sort_by(|(a, _), (b, _)| {
        (joker_type(a), a.map(JokerCard::from)).cmp(&(joker_type(b), b.map(JokerCard::from)))
    });
    hands
        .iter()
        .enumerate()
        .fold(0, |acc, (i, (_, bid))| acc + (i as u32 + 1) * bid)
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let input = read(args.path.as_path()).unwrap();
    let input = std::str::from_utf8(&input).unwrap();
    let (_, input) = camel_card(input).unwrap();
    println!("Part 1: {}", total_winnings(input.clone()));
    println!("Part 2: {}", joker_total_winnings(input.clone()));
    println!("Time elapsed: {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input() {
        let (remainder, hands) = camel_card(
            "32T3K 765
                   T55J5 684
                   KK677 28
                   KTJJT 220
                   QQQJA 483",
        )
        .unwrap();
        assert_eq!(remainder, "");
        assert_eq!(
            hands,
            vec![
                (
                    [
                        Card::Number(3),
                        Card::Number(2),
                        Card::T,
                        Card::Number(3),
                        Card::K
                    ],
                    765
                ),
                (
                    [
                        Card::T,
                        Card::Number(5),
                        Card::Number(5),
                        Card::J,
                        Card::Number(5)
                    ],
                    684
                ),
                (
                    [
                        Card::K,
                        Card::K,
                        Card::Number(6),
                        Card::Number(7),
                        Card::Number(7)
                    ],
                    28
                ),
                ([Card::K, Card::T, Card::J, Card::J, Card::T], 220),
                ([Card::Q, Card::Q, Card::Q, Card::J, Card::A], 483),
            ]
        );
    }

    #[test]
    fn test_compare_hand_type() {
        assert!(HandType::None < HandType::HighCard);
        assert!(HandType::HighCard < HandType::OnePair);
        assert!(HandType::OnePair < HandType::TwoPair);
        assert!(HandType::TwoPair < HandType::ThreeOfAKind);
        assert!(HandType::ThreeOfAKind < HandType::FullHouse);
        assert!(HandType::FullHouse < HandType::FourOfAKind);
        assert!(HandType::FourOfAKind < HandType::FiveOfAKind);
    }

    #[test]
    fn test_example() {
        let input = "32T3K 765
                     T55J5 684
                     KK677 28
                     KTJJT 220
                     QQQJA 483";
        let (_, hands) = camel_card(input).unwrap();
        assert_eq!(total_winnings(hands), 6440);
    }

    #[test]
    fn test_joker_type() {
        let input = "32T3K 765
                     T55J5 684
                     KK677 28
                     KTJJT 220
                     QQQJA 483";
        let (_, hands) = camel_card(input).unwrap();
        assert_eq!(joker_type(&hands[0].0), HandType::OnePair);
        assert_eq!(joker_type(&hands[1].0), HandType::FourOfAKind);
        assert_eq!(joker_type(&hands[2].0), HandType::TwoPair);
        assert_eq!(joker_type(&hands[3].0), HandType::FourOfAKind);
        assert_eq!(joker_type(&hands[4].0), HandType::FourOfAKind);
    }

    #[test]
    fn test_joker_total_winnings() {
        let input = "32T3K 765
                     T55J5 684
                     KK677 28
                     KTJJT 220
                     QQQJA 483";
        let (_, hands) = camel_card(input).unwrap();
        assert_eq!(joker_total_winnings(hands), 5905);
    }
}
