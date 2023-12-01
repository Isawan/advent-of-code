use clap::{arg, command, Parser};
use regex::Regex;
use std::sync::OnceLock;
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

fn first_digit(s: &str) -> Option<u32> {
    s.chars()
        .find(|c| c.is_ascii_digit())?
        .to_string()
        .parse::<u32>()
        .ok()
}
fn last_digit(s: &str) -> Option<u32> {
    s.chars()
        .filter(|c| c.is_ascii_digit())
        .last()?
        .to_string()
        .parse::<u32>()
        .ok()
}

fn number_digit(s: &str) -> Option<u32> {
    let first = first_digit(s)?;
    let last = last_digit(s)?;
    format!("{}{}", first, last).parse::<u32>().ok()
}

fn forward_to_digit(s: &str) -> Option<u32> {
    match s {
        "one" | "1" => Some(1),
        "two" | "2" => Some(2),
        "three" | "3" => Some(3),
        "four" | "4" => Some(4),
        "five" | "5" => Some(5),
        "six" | "6" => Some(6),
        "seven" | "7" => Some(7),
        "eight" | "8" => Some(8),
        "nine" | "9" => Some(9),
        _ => None,
    }
}

fn backwards_to_digit(s: &str) -> Option<u32> {
    let reverse = s.chars().rev().collect::<String>();
    forward_to_digit(&reverse)
}

static FORWARD: OnceLock<Regex> = OnceLock::new();
static BACKWARD: OnceLock<Regex> = OnceLock::new();

fn word_digit(line: &str) -> Option<u32> {
    let forward_search = FORWARD.get_or_init(|| {
        Regex::new(r"(one|two|three|four|five|six|seven|eight|nine|1|2|3|4|5|6|7|8|9)").unwrap()
    });
    let backwards_search = BACKWARD.get_or_init(|| {
        Regex::new(r"(eno|owt|eerht|ruof|evif|xis|neves|thgie|enin|1|2|3|4|5|6|7|8|9)").unwrap()
    });

    // handle first digit
    let capture = forward_search.captures(line)?;
    let word = capture.get(1)?;
    let first = forward_to_digit(word.as_str())?;

    // handle last digit
    let reverse_line = line.chars().rev().collect::<String>();
    let capture = backwards_search.captures(&reverse_line)?;
    let word = capture.get(1)?;
    let last = backwards_to_digit(word.as_str())?;

    format!("{}{}", first, last).parse::<u32>().ok()
}

fn main() {
    let args = Cli::parse();
    let input = File::open(args.path.as_path()).unwrap();
    let reader = BufReader::new(&input);
    let sum = reader
        .lines()
        .map(|line| number_digit(&line.unwrap()))
        .collect::<Option<Vec<u32>>>()
        .expect("Error occured")
        .iter()
        .sum::<u32>();
    println!("{:?}", sum);

    let input = File::open(args.path.as_path()).unwrap();
    let reader = BufReader::new(&input);
    let sum = reader
        .lines()
        .map(|line| word_digit(&line.unwrap()))
        .collect::<Option<Vec<u32>>>()
        .expect("Error occured")
        .iter()
        .sum::<u32>();
    println!("{:?}", sum);
}
