use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Clone, Debug)]
enum Shape {
    Rock,
    Paper,
    Scissor,
}

#[derive(Clone, Debug)]
struct Line {
    opponent: Shape,
    mine: Shape,
}

#[allow(dead_code)]
fn parse(line: &str) -> Line {
    let mut split = line.split(' ');
    Line {
        opponent: match split.next().unwrap() {
            "A" => Shape::Rock,
            "B" => Shape::Paper,
            "C" => Shape::Scissor,
            _ => panic!("Unexpected"),
        },
        mine: match split.next().unwrap() {
            "X" => Shape::Rock,
            "Y" => Shape::Paper,
            "Z" => Shape::Scissor,
            _ => panic!("Unexpected"),
        },
    }
}

fn wins(shape: Shape) -> Shape {
    match shape {
        Shape::Rock => Shape::Paper,
        Shape::Paper => Shape::Scissor,
        Shape::Scissor => Shape::Rock,
    }
}
fn loses(shape: Shape) -> Shape {
    match shape {
        Shape::Rock => Shape::Scissor,
        Shape::Paper => Shape::Rock,
        Shape::Scissor => Shape::Paper,
    }
}

fn score(line: Line) -> usize {
    let outcome_score = match line {
        Line {
            opponent: Shape::Rock,
            mine: Shape::Rock,
        } => 3,
        Line {
            opponent: Shape::Rock,
            mine: Shape::Paper,
        } => 6,
        Line {
            opponent: Shape::Rock,
            mine: Shape::Scissor,
        } => 0,
        Line {
            opponent: Shape::Paper,
            mine: Shape::Rock,
        } => 0,
        Line {
            opponent: Shape::Paper,
            mine: Shape::Paper,
        } => 3,
        Line {
            opponent: Shape::Paper,
            mine: Shape::Scissor,
        } => 6,
        Line {
            opponent: Shape::Scissor,
            mine: Shape::Rock,
        } => 6,
        Line {
            opponent: Shape::Scissor,
            mine: Shape::Paper,
        } => 0,
        Line {
            opponent: Shape::Scissor,
            mine: Shape::Scissor,
        } => 3,
    };
    let shape_score = match line.mine {
        Shape::Rock => 1,
        Shape::Paper => 2,
        Shape::Scissor => 3,
    };
    outcome_score + shape_score
}

fn parse_two(line: &str) -> Line { 
    let mut split = line.split(' ');
    let opponent = match split.next().unwrap() {
            "A" => Shape::Rock,
            "B" => Shape::Paper,
            "C" => Shape::Scissor,
            _ => panic!("Unexpected"),
        };
    let x = Line {
        opponent: opponent.clone(),
        mine: match split.next().unwrap() {
            "X" => loses(opponent),
            "Y" => opponent,
            "Z" => wins(opponent),
            _ => panic!("Unexpected"),
        },
    };
    println!("{:?}", x);
    x
}

fn main() {
    let args = Cli::from_args();
    let input = File::open(args.path.as_path()).unwrap();
    let lines = BufReader::new(input).lines();
    //let first_score = lines.map(|x| parse(&x.unwrap())).fold(0,|x,y| score(y)+x);
    let second_score = lines.map(|x| parse_two(&x.unwrap())).fold(0,|x,y| score(y)+x);
    println!("{}", second_score);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {}
}
