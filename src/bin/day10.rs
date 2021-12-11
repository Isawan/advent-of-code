use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use structopt::StructOpt;
#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn parse_corrupt(source: &str) -> Option<char> {
    let mut stack = Vec::new();
    for (i, c) in source.chars().enumerate() {
        match c {
            '(' | '[' | '{' | '<' => {
                stack.push(c);
            }
            ')' | ']' | '}' | '>' => {
                let last = stack.pop().unwrap();
                let expected = match c {
                    ')' => '(',
                    ']' => '[',
                    '}' => '{',
                    '>' => '<',
                    _ => {
                        panic!("unexpected")
                    }
                };
                if last != expected {
                    return Some(c);
                }
            }
            _ => {
                panic!("Unexpected")
            }
        }
    }
    None
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let illegal_score = source
        .trim()
        .split('\n')
        .filter_map(|line| parse_corrupt(line).map(|x| match x {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
        _ => panic!("Unexpected"),
        })).fold(0, |a,x| a+x);
    println!("illegal score = {}", illegal_score);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_corrupt() {
        let t1 = "{([(<{}[<>[]}>{[]{[(<()>";
        let p = parse_corrupt(t1).unwrap();
        assert_eq!(p, '}');
        let t2 = "[[<[([]))<([[{}[[()]]]";
        let p = parse_corrupt(t2).unwrap();
        assert_eq!(p, ')');
        let t3 = "[{[{({}]{}}([{[{{{}}([]";
        let p = parse_corrupt(t3).unwrap();
        assert_eq!(p, ']');
        let t4 = "[<(<(<(<{}))><([]([]()";
        let p = parse_corrupt(t4).unwrap();
        assert_eq!(p, ')');
        let t5 = "<{([([[(<>()){}]>(<<{{";
        let p = parse_corrupt(t5).unwrap();
        assert_eq!(p, '>');
    }
}
