use std::fs;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

enum Line {
    Corrupt(char),
    Incomplete(Vec<char>),
}

fn score_incomplete_sequence(sequence: Vec<char>) -> u64 {
    sequence.iter().fold(0, |a, x| {
        (a * 5)
            + match x {
                ')' => 1,
                ']' => 2,
                '}' => 3,
                '>' => 4,
                _ => {
                    panic!("Unexpected character")
                }
            }
    })
}

fn parse_corrupt(source: &str) -> Line {
    let mut stack = Vec::new();
    for c in source.chars() {
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
                    return Line::Corrupt(c);
                }
            }
            _ => {
                panic!("Unexpected")
            }
        }
    }
    stack.reverse();
    Line::Incomplete(
        stack
            .iter()
            .map(|x| match x {
                '(' => ')',
                '[' => ']',
                '{' => '}',
                '<' => '>',
                _ => panic!("Unexpected char"),
            })
            .collect(),
    )
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let illegal_score = source
        .trim()
        .split('\n')
        .filter_map(|line| match parse_corrupt(line) {
            Line::Corrupt(c) => Some(c),
            Line::Incomplete(_) => None,
        })
        .map(|x| match x {
            ')' => 3,
            ']' => 57,
            '}' => 1197,
            '>' => 25137,
            _ => panic!("Unexpected"),
        })
        .fold(0, |a, x| a + x);
    println!("illegal score = {}", illegal_score);

    let mut incomplete_scores = source
        .trim()
        .split('\n')
        .filter_map(|line| match parse_corrupt(line) {
            Line::Corrupt(_) => None,
            Line::Incomplete(c) => Some(score_incomplete_sequence(c)),
        })
        .collect::<Vec<u64>>();
    incomplete_scores.sort();
    let total_incomplete_score = incomplete_scores
        .get((incomplete_scores.len() - 1) / 2)
        .unwrap();
    println!("total incomplete score: {}", total_incomplete_score);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_corrupt() {
        let t1 = "{([(<{}[<>[]}>{[]{[(<()>";
        if let Line::Corrupt(p) = parse_corrupt(t1) {
            assert_eq!(p, '}');
        } else {
            panic!("Expected corrupt line");
        }
        let t2 = "[[<[([]))<([[{}[[()]]]";
        if let Line::Corrupt(p) = parse_corrupt(t2) {
            assert_eq!(p, ')');
        } else {
            panic!("Expected corrupt line");
        }
        let t3 = "[{[{({}]{}}([{[{{{}}([]";
        if let Line::Corrupt(p) = parse_corrupt(t3) {
            assert_eq!(p, ']');
        } else {
            panic!("Expected corrupt line");
        }
        let t4 = "[<(<(<(<{}))><([]([]()";
        if let Line::Corrupt(p) = parse_corrupt(t4) {
            assert_eq!(p, ')');
        } else {
            panic!("Expected corrupt line");
        }
        let t5 = "<{([([[(<>()){}]>(<<{{";
        if let Line::Corrupt(p) = parse_corrupt(t5) {
            assert_eq!(p, '>');
        } else {
            panic!("Expected corrupt line");
        }
    }

    #[test]
    fn test_incomplete_scorer() {
        let t = "])}>".chars().collect();
        let score = score_incomplete_sequence(t);
        assert_eq!(score, 294);
    }

    #[test]
    fn test_incomplete_full() {
        let lines = &[
            (
                "[({(<(())[]>[[{[]{<()<>>",
                "}}]])})]".chars().collect::<Vec<char>>(),
                288957,
            ),
            (
                "[(()[<>])]({[<{<<[]>>(",
                ")}>]})".chars().collect::<Vec<char>>(),
                5566,
            ),
            (
                "(((({<>}<{<{<>}{[]{[]{}",
                "}}>}>))))".chars().collect::<Vec<char>>(),
                1480781,
            ),
            (
                "{<[[]]>}<{[{[{[]{()[[[]",
                "]]}}]}]}>".chars().collect::<Vec<char>>(),
                995444,
            ),
            (
                "<{([{{}}[<[[[<>{}]]]>[]]",
                "])}>".chars().collect::<Vec<char>>(),
                294,
            ),
        ];
        let mut scores = Vec::new();
        for (line, ex_compl, ex_points) in lines {
            if let Line::Incomplete(compl) = parse_corrupt(line) {
                assert_eq!(compl, *ex_compl);
                let points = score_incomplete_sequence(compl);
                assert_eq!(points, *ex_points);
                scores.push(points);
            } else {
                panic!("not expected")
            }
        }
        scores.sort();
        println!("{:?}", scores);
        let total = scores.get((scores.len() - 1) / 2).unwrap();
        assert_eq!(*total, 288957);
    }
}
