use clap::Parser;
use nom::{
    character::complete::{newline, space0},
    combinator::opt,
    multi::many1,
    sequence::terminated,
    IResult,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn sequence(input: &str) -> IResult<&str, Vec<i32>> {
    many1(terminated(nom::character::complete::i32, space0))(input)
}

fn sequences(input: &str) -> IResult<&str, Vec<Vec<i32>>> {
    many1(terminated(sequence, opt(newline)))(input)
}

fn lower_increment(input: &[i32]) -> i32 {
    assert_ne!(input.len(), 0);
    if input.iter().all(|d| *d == 0) {
        0
    } else {
        let next: Vec<_> = input.windows(2).map(|w| w[1] - w[0]).collect();
        input.last().unwrap() + lower_increment(&next)
    }
}

fn front_lower_increment(input: &[i32]) -> i32 {
    if input.iter().all(|d| *d == 0) {
        0
    } else {
        let next: Vec<_> = input.windows(2).map(|w| w[1] - w[0]).collect();
        input.first().unwrap() - front_lower_increment(&next)
    }
}

fn part1(input: &[Vec<i32>]) -> i32 {
    input.iter().map(|s| lower_increment(s)).sum()
}

fn part2(input: &[Vec<i32>]) -> i32 {
    input.iter().map(|s| front_lower_increment(s)).sum()
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    let (_, sequences) = sequences(&input).unwrap();
    println!("Part 1: {}", part1(&sequences));
    println!("Part 2: {}", part2(&sequences));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line_parse() {
        let input = "0 3 6 9 12 15";
        let (_, output) = super::sequences(input).unwrap();
        assert_eq!(output, vec![vec![0, 3, 6, 9, 12, 15]])
    }

    #[test]
    fn test_single_line_parse_with_newline() {
        let input = "0 3 6 9 12 15\n";
        let (_, output) = super::sequences(input).unwrap();
        assert_eq!(output, vec![vec![0, 3, 6, 9, 12, 15]])
    }

    #[test]
    fn test_multiline_parse() {
        let input = "0 3 6 9 12 15\n0 3 6 9 12 15\n";
        let (_, output) = super::sequences(input).unwrap();
        assert_eq!(
            output,
            vec![vec![0, 3, 6, 9, 12, 15], vec![0, 3, 6, 9, 12, 15]]
        )
    }

    #[test]
    fn test_example_single() {
        let input = [0, 3, 6, 9, 12, 15];
        assert_eq!(lower_increment(&input), 18)
    }

    #[test]
    fn test_full_example() {
        let input = "0 3 6 9 12 15\n1 3 6 10 15 21\n10 13 16 21 30 45";
        let (_, output) = sequences(input).unwrap();
        println!("{:?}", output);
        assert_eq!(part1(&output), 114)
    }

    #[test]
    fn test_front_lower_increment() {
        let input = [10, 13, 16, 21, 30, 45];

        assert_eq!(front_lower_increment(&input), 5)
    }

    #[test]
    fn test_part2_example() {
        let input = "0 3 6 9 12 15\n1 3 6 10 15 21\n10 13 16 21 30 45";
        let (_, output) = sequences(input).unwrap();
        assert_eq!(part2(&output), 2)
    }
}
