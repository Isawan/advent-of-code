use itertools::iproduct;
use std::cmp::max;
use std::collections::BTreeSet;
use std::iter::zip;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Knots {
    head: (i32, i32),
    tail: (i32, i32),
}

fn move_tail(knots: Knots) -> Knots {
    let xdiff = knots.head.0 - knots.tail.0;
    let ydiff = knots.head.1 - knots.tail.1;
    Knots {
        head: knots.head,
        tail: (
            knots.tail.0
                + if xdiff.abs() > 1 || ydiff.abs() > 1 {
                    xdiff.signum()
                } else {
                    0
                },
            knots.tail.1
                + if xdiff.abs() > 1 || ydiff.abs() > 1 {
                    ydiff.signum()
                } else {
                    0
                },
        ),
    }
}

fn perform(knots: Knots, dir: &str) -> Knots {
    let new_head = match dir {
        "U" => (knots.head.0, knots.head.1 + 1),
        "D" => (knots.head.0, knots.head.1 - 1),
        "R" => (knots.head.0 + 1, knots.head.1),
        "L" => (knots.head.0 - 1, knots.head.1),
        _ => panic!("unexpected"),
    };
    move_tail(Knots {
        head: new_head,
        tail: knots.tail,
    })
}

fn calc(lines: &str) -> usize {
    lines
        .lines()
        .fold(
            (
                BTreeSet::new(),
                Knots {
                    head: (0, 0),
                    tail: (0, 0),
                },
            ),
            |(mut previous_tail_positions, mut current), line| {
                let parts = line.split(" ").collect::<Vec<&str>>();
                match parts.as_slice() {
                    [direction, mut times] => {
                        for _ in 0..(times.parse::<usize>().unwrap()) {
                            current = perform(current, direction);
                            previous_tail_positions.insert((current.tail));
                        }
                        (previous_tail_positions, current)
                    }
                    _ => panic!(""),
                }
            },
        )
        .0
        .len()
}

fn move_general(head: (i32, i32), tail: (i32, i32)) -> (i32, i32) {
    let xdiff = head.0 - tail.0;
    let ydiff = head.1 - tail.1;
    (
        tail.0
            + if xdiff.abs() > 1 || ydiff.abs() > 1 {
                xdiff.signum()
            } else {
                0
            },
        tail.1
            + if xdiff.abs() > 1 || ydiff.abs() > 1 {
                ydiff.signum()
            } else {
                0
            },
    )
}

fn perform_general(knots: Vec<(i32, i32)>, dir: &str) -> Vec<(i32, i32)> {
    let mut new_head = match dir {
        "U" => (knots[0].0, knots[0].1 + 1),
        "D" => (knots[0].0, knots[0].1 - 1),
        "R" => (knots[0].0 + 1, knots[0].1),
        "L" => (knots[0].0 - 1, knots[0].1),
        _ => panic!("unexpected"),
    };
    let mut new_rope = Vec::new();
    new_rope.push(new_head);
    for tail in knots.iter().skip(1) {
        new_head = move_general(new_head, *tail);
        new_rope.push(new_head);
    }
    new_rope
}

fn calc2(lines: &str) -> usize {
    let mut previous_tails = BTreeSet::new();
    previous_tails.insert((0, 0));
    lines
        .lines()
        .fold(
            (previous_tails, vec![(0, 0); 10]),
            |(mut previous_tails, mut current), line| {
                let parts = line.split(" ").collect::<Vec<&str>>();
                match parts.as_slice() {
                    [direction, mut times] => {
                        for _ in 0..(times.parse::<usize>().unwrap()) {
                            current = perform_general(current, direction);
                            previous_tails.insert(*(current.last().unwrap()));
                        }
                        (previous_tails, current)
                    }
                    _ => panic!("Error"),
                }
            },
        )
        .0
        .len()
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    println!("{:?}", calc(&input));
    println!("{:?}", calc2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_tail() {
        assert_eq!(
            move_tail(Knots {
                head: (2, 0),
                tail: (0, 0)
            }),
            Knots {
                head: (2, 0),
                tail: (1, 0)
            }
        );
        assert_eq!(
            move_tail(Knots {
                head: (2, 2),
                tail: (0, 0)
            }),
            Knots {
                head: (2, 2),
                tail: (1, 1)
            }
        );
        assert_eq!(
            move_tail(Knots {
                head: (1, 0),
                tail: (0, 0)
            }),
            Knots {
                head: (1, 0),
                tail: (0, 0)
            }
        );
        assert_eq!(
            move_tail(Knots {
                head: (4, 2),
                tail: (3, 0)
            }),
            Knots {
                head: (4, 2),
                tail: (4, 1)
            }
        );
    }

    #[test]
    fn test_case() {
        let input = include_str!("../../input/day9-test");
        assert_eq!(calc(input), 13);
    }

    #[test]
    fn test_move_tail_general() {
        assert_eq!(move_general((2, 0), (0, 0)), (1, 0));
        assert_eq!(move_general((2, 2), (0, 0)), (1, 1));
        assert_eq!(move_general((1, 0), (0, 0)), (0, 0));
        assert_eq!(move_general((4, 2), (3, 0)), (4, 1));
    }

    #[test]
    fn test_case_general() {
        let input = include_str!("../../input/day9-test");
        assert_eq!(calc2(input), 1);
        let input = include_str!("../../input/day9-test2");
        assert_eq!(calc2(input), 36);
    }
}
