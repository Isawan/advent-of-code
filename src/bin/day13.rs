use regex::Regex;
use std::cmp::max;
use std::collections::BTreeSet;
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FoldInstruction {
    Left(i32),
    Up(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Dot(i32, i32);

fn parse_dots(source: &str) -> BTreeSet<Dot> {
    let mut v = BTreeSet::new();
    for line in source.trim().split('\n') {
        let mut nums = line.split(',');
        v.insert(Dot(
            nums.next().unwrap().parse().unwrap(),
            nums.next().unwrap().parse().unwrap(),
        ));
    }
    v
}

fn parse_folding_instructions(source: &str) -> Vec<FoldInstruction> {
    let mut v = Vec::new();
    let re = Regex::new(r"fold along (x|y)=(\d+)").unwrap();
    for line in source.trim().split('\n') {
        let capture = re.captures(line).unwrap();
        let p = capture.get(2).unwrap().as_str().parse().unwrap();
        v.push(match capture.get(1).unwrap().as_str() {
            "x" => FoldInstruction::Left(p),
            "y" => FoldInstruction::Up(p),
            _ => {
                panic!("invalid direction")
            }
        })
    }
    v
}

fn fold(ins: FoldInstruction, dots: &BTreeSet<Dot>) -> BTreeSet<Dot> {
    dots.iter()
        .map(|dot| match ins {
            FoldInstruction::Left(i) => Dot(if dot.0 < i { dot.0 } else { i - (dot.0 - i) }, dot.1),
            FoldInstruction::Up(i) => Dot(dot.0, if dot.1 < i { dot.1 } else { i - (dot.1 - i) }),
        })
        .collect()
}

fn parse_input(source: &str) -> (BTreeSet<Dot>, Vec<FoldInstruction>) {
    let re = Regex::new(r"\n\n").unwrap();
    let i = re.find(source).unwrap().start();
    let (dot_source, fold_source) = source.split_at(i);
    (
        parse_dots(&dot_source),
        parse_folding_instructions(&fold_source[2..]),
    )
}

fn visualize_dots(dots: &BTreeSet<Dot>) -> String {
    let max_x = dots.iter().fold(0, |a, dot| max(a, dot.0));
    let max_y = dots.iter().fold(0, |a, dot| max(a, dot.1));
    let mut s = String::new();
    for y in 0..max_y + 1 {
        for x in 0..max_x + 1 {
            s.push(if dots.contains(&Dot(x, y)) { '#' } else { ' ' });
        }
        s.push('\n');
    }
    s
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let (mut dots, fi) = parse_input(&source);
    let first_fold = fold(fi[0], &dots);
    println!("dots after first fold: {}", first_fold.len());

    for f in fi {
        dots = fold(f, &dots);
    }
    println!("{}", visualize_dots(&dots));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dots() {
        let input = "6,10\n\
                     9,14\n\
                     1,1\n";
        let dots = parse_dots(input);
        assert!(dots.contains(&Dot(6, 10)));
        assert!(dots.contains(&Dot(9, 14)));
        assert!(dots.contains(&Dot(1, 1)));
    }

    #[test]
    fn test_folding_instructions() {
        let input = "fold along y=70\n\
                    fold along x=5\n";
        let fi = parse_folding_instructions(input);
        assert_eq!(fi[0], FoldInstruction::Up(70));
        assert_eq!(fi[1], FoldInstruction::Left(5));
    }

    #[test]
    fn test_full_parser() {
        let input = "6,10\n\
                     9,14\n\
                     1,1\n\
                     \n\
                     fold along y=70\n\
                     fold along x=5\n";
        let (dots, fi) = parse_input(input);
        assert!(dots.contains(&Dot(6, 10)));
        assert!(dots.contains(&Dot(9, 14)));
        assert!(dots.contains(&Dot(1, 1)));
        assert_eq!(fi[0], FoldInstruction::Up(70));
        assert_eq!(fi[1], FoldInstruction::Left(5));
    }

    #[test]
    fn test_left_fold() {
        let mut input = BTreeSet::new();
        input.insert(Dot(6, 10));
        input.insert(Dot(9, 14));
        let mut exp = BTreeSet::new();
        exp.insert(Dot(6, 10));
        exp.insert(Dot(7, 14));
        let output = fold(FoldInstruction::Left(8), &input);
        assert_eq!(output, exp);
    }

    #[test]
    fn test_up_fold() {
        let mut input = BTreeSet::new();
        input.insert(Dot(6, 10));
        input.insert(Dot(9, 14));
        let mut exp = BTreeSet::new();
        exp.insert(Dot(6, 10));
        exp.insert(Dot(9, 10));
        let output = fold(FoldInstruction::Up(12), &input);
        assert_eq!(output, exp);
    }

    #[test]
    fn test_full_fold() {
        let input = "6,10\n\
                     0,14\n\
                     9,10\n\
                     0,3\n\
                     10,4\n\
                     4,11\n\
                     6,0\n\
                     6,12\n\
                     4,1\n\
                     0,13\n\
                     10,12\n\
                     3,4\n\
                     3,0\n\
                     8,4\n\
                     1,10\n\
                     2,14\n\
                     8,10\n\
                     9,0\n\
                     \n\
                     fold along y=7\n\
                     fold along x=5\n";
        let (dots, fi) = parse_input(input);
    }
}
