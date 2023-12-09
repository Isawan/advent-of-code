use std::{fs::read, time::Instant};

use clap::Parser;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::{map, map_res},
    error::ParseError,
    multi::separated_list1,
    sequence::{delimited, pair, separated_pair},
    IResult,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
struct MaxConstraint {
    red: u32,
    green: u32,
    blue: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Reveal {
    red: u32,
    green: u32,
    blue: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Game {
    id: u32,
    reveals: Vec<Reveal>,
}

#[derive(Debug, PartialEq, Eq)]
enum Color {
    Red,
    Blue,
    Green,
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn number(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(input)
}

#[allow(clippy::type_complexity)]
fn game(input: &str) -> IResult<&str, (u32, Vec<Vec<(u32, Color)>>)> {
    ws(pair(delimited(tag("Game "), number, tag(": ")), reveals))(input)
}

fn reveals(input: &str) -> IResult<&str, Vec<Vec<(u32, Color)>>> {
    separated_list1(tag("; "), reveal)(input)
}

fn reveal(input: &str) -> IResult<&str, Vec<(u32, Color)>> {
    separated_list1(tag(", "), separated_pair(number, tag(" "), color))(input)
}

fn color(input: &str) -> IResult<&str, Color> {
    alt((
        map(tag("red"), |_| Color::Red),
        map(tag("blue"), |_| Color::Blue),
        map(tag("green"), |_| Color::Green),
    ))(input)
}

fn interpret_reveal(reveal: &[(u32, Color)]) -> Reveal {
    reveal.iter().fold(
        Reveal {
            red: 0,
            green: 0,
            blue: 0,
        },
        |acc, (n, color)| match color {
            Color::Red => Reveal {
                red: acc.red + n,
                ..acc
            },
            Color::Green => Reveal {
                green: acc.green + n,
                ..acc
            },
            Color::Blue => Reveal {
                blue: acc.blue + n,
                ..acc
            },
        },
    )
}

fn interpret_game(game: (u32, Vec<Vec<(u32, Color)>>)) -> Game {
    let (id, reveals) = game;
    Game {
        id,
        reveals: reveals.iter().map(|x| interpret_reveal(x)).collect(),
    }
}

fn possible_game(game: &Game, constraint: &MaxConstraint) -> bool {
    game.reveals.iter().all(|reveal| {
        reveal.red <= constraint.red
            && reveal.green <= constraint.green
            && reveal.blue <= constraint.blue
    })
}

fn max_possible_cubes(game: &Game) -> MaxConstraint {
    let constraint = MaxConstraint {
        red: 0,
        blue: 0,
        green: 0,
    };
    game.reveals
        .iter()
        .fold(constraint, |acc, reveal| MaxConstraint {
            red: std::cmp::max(acc.red, reveal.red),
            blue: std::cmp::max(acc.blue, reveal.blue),
            green: std::cmp::max(acc.green, reveal.green),
        })
}

fn min_sum_power(input: &str) -> u32 {
    input
        .lines()
        .map(|g| game(g).expect("could not parse").1)
        .map(interpret_game)
        .fold(0, |acc, g| {
            let MaxConstraint { red, blue, green } = max_possible_cubes(&g);
            acc + (red * blue * green)
        })
}

fn id_sum(input: &str, constraint: &MaxConstraint) -> u32 {
    input
        .lines()
        .map(|g| game(g).expect("could not parse").1)
        .map(interpret_game)
        .filter(|g| possible_game(g, constraint))
        .fold(0, |acc, g| acc + g.id)
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let input = read(args.path.as_path()).unwrap();
    let constraint = MaxConstraint {
        red: 12,
        green: 13,
        blue: 14,
    };

    println!(
        "Part 1: {}",
        id_sum(std::str::from_utf8(&input).unwrap(), &constraint)
    );
    println!(
        "Part 2: {}",
        min_sum_power(std::str::from_utf8(&input).unwrap())
    );

    println!("Time elapsed: {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game() {
        let input = "Game 1: 1 red, 1 blue, 1 green; 2 red, 2 blue, 2 green";
        let expected = (
            1,
            vec![
                vec![(1, Color::Red), (1, Color::Blue), (1, Color::Green)],
                vec![(2, Color::Red), (2, Color::Blue), (2, Color::Green)],
            ],
        );
        assert_eq!(game(input), Ok(("", expected)));
    }

    #[test]
    fn test_reveal() {
        let input = "1 red, 1 blue, 1 green";
        let expected = vec![(1, Color::Red), (1, Color::Blue), (1, Color::Green)];
        assert_eq!(reveal(input), Ok(("", expected)));
    }

    #[test]
    fn test_reveals() {
        let input = "1 red, 1 blue, 1 green; 2 red, 2 blue, 2 green";
        let expected = vec![
            vec![(1, Color::Red), (1, Color::Blue), (1, Color::Green)],
            vec![(2, Color::Red), (2, Color::Blue), (2, Color::Green)],
        ];
        assert_eq!(reveals(input), Ok(("", expected)));
    }

    #[test]
    fn test_interpret_reveal() {
        let input = vec![(2, Color::Red), (1, Color::Blue), (1, Color::Green)];
        let expected = Reveal {
            red: 2,
            green: 1,
            blue: 1,
        };
        assert_eq!(interpret_reveal(&input), expected);
    }

    #[test]
    fn test_interpret_reveal_with_missing_color() {
        let input = vec![(2, Color::Red), (1, Color::Blue)];
        let expected = Reveal {
            red: 2,
            green: 0,
            blue: 1,
        };
        assert_eq!(interpret_reveal(&input), expected);
    }

    #[test]
    fn test_example() {
        let constraint = MaxConstraint {
            red: 12,
            green: 13,
            blue: 14,
        };
        let example = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
                       Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
                       Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
                       Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
                       Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";
        let id_sum = id_sum(example, &constraint);
        assert_eq!(id_sum, 8)
    }

    #[test]
    fn min_max_constraint() {
        let example = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
        // parse
        let (_, game) = game(example).unwrap();
        let game = interpret_game(game);
        let power = max_possible_cubes(&game);
        assert_eq!(
            power,
            MaxConstraint {
                red: 4,
                green: 2,
                blue: 6,
            }
        )
    }

    #[test]
    fn test_min_sum_power() {
        let example = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
                       Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
                       Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
                       Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
                       Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";
        let min_sum_power = min_sum_power(example);
        assert_eq!(min_sum_power, 2286)
    }
}
