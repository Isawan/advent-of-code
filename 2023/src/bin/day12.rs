use std::collections::HashMap;

use ahash::RandomState;
use bumpalo::Bump;
use clap::Parser;
use nom::{
    branch::alt,
    character::complete::space1,
    combinator::{opt, value},
    multi::many1,
    sequence::{separated_pair, terminated},
    IResult,
};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum SpringCondition {
    Operational,
    Damaged,
    Unknown,
}

fn record(input: &str) -> IResult<&str, (Vec<SpringCondition>, Vec<u32>)> {
    separated_pair(
        many1(alt((
            value(
                SpringCondition::Damaged,
                nom::character::complete::char('#'),
            ),
            value(
                SpringCondition::Operational,
                nom::character::complete::char('.'),
            ),
            value(
                SpringCondition::Unknown,
                nom::character::complete::char('?'),
            ),
        ))),
        space1,
        many1(terminated(
            nom::character::complete::u32,
            opt(nom::character::complete::char(',')),
        )),
    )(input)
}

#[allow(clippy::type_complexity)]
fn records(input: &str) -> IResult<&str, Vec<(Vec<SpringCondition>, Vec<u32>)>> {
    many1(terminated(record, opt(nom::character::complete::newline)))(input)
}
fn valid_combos<'a>(
    alloc: &'a Bump,
    mem: &mut HashMap<(&'a [SpringCondition], &'a [u32]), u64, RandomState>,
    (conditions, criteria): (&[SpringCondition], &[u32]),
) -> u64 {
    if let Some(result) = mem.get(&(conditions, criteria)) {
        return *result;
    }
    let result = match (conditions, criteria) {
        ([], []) => 1,
        ([], [_, ..]) => 0,
        ([SpringCondition::Damaged], []) => 0,
        ([SpringCondition::Damaged], [1]) => 1,
        ([SpringCondition::Damaged], [_]) => 0,
        ([SpringCondition::Damaged], [_, _, ..]) => 0,
        ([SpringCondition::Operational], []) => 1,
        ([SpringCondition::Operational], [_]) => 0,
        ([SpringCondition::Operational, remain_cond @ ..], []) => {
            valid_combos(alloc, mem, (remain_cond, &[]))
        }
        ([SpringCondition::Operational, remain_cond @ ..], [..]) => {
            valid_combos(alloc, mem, (&remain_cond, &criteria))
        }
        ([SpringCondition::Damaged, SpringCondition::Operational, ..], [1, remain_crit @ ..]) => {
            valid_combos(alloc, mem, (&conditions[1..], remain_crit))
        }
        ([SpringCondition::Damaged, SpringCondition::Operational, ..], [_, ..]) => 0,
        ([SpringCondition::Damaged, SpringCondition::Damaged, ..], [0, ..]) => 0,
        ([SpringCondition::Damaged, SpringCondition::Damaged, ..], [first, remain_crit @ ..]) => {
            let mut next = Vec::new();
            next.push(first - 1);
            next.extend(remain_crit);
            valid_combos(alloc, mem, (&conditions[1..], &next))
        }
        ([SpringCondition::Damaged, _, ..], []) => 0,
        ([SpringCondition::Damaged, SpringCondition::Unknown, remain_cond @ ..], _) => {
            let mut left = Vec::new();
            let mut right = Vec::new();
            left.push(SpringCondition::Damaged);
            left.push(SpringCondition::Damaged);
            left.extend(remain_cond);
            right.push(SpringCondition::Damaged);
            right.push(SpringCondition::Operational);
            right.extend(remain_cond);
            valid_combos(alloc, mem, (&left, criteria))
                + valid_combos(alloc, mem, (&right, criteria))
        }
        ([SpringCondition::Unknown, remain_cond @ ..], _) => {
            let mut left = Vec::new();
            let mut right = Vec::new();
            left.push(SpringCondition::Operational);
            left.extend(remain_cond);
            right.push(SpringCondition::Damaged);
            right.extend(remain_cond);
            valid_combos(alloc, mem, (&left, criteria))
                + valid_combos(alloc, mem, (&right, criteria))
        }
    };
    let condition_key = alloc.alloc_slice_clone(conditions);
    let criteria_key = alloc.alloc_slice_clone(criteria);
    mem.insert((condition_key, criteria_key), result);
    result
}

fn part1(input: &str) -> u64 {
    let (_, records) = records(input).unwrap();
    records
        .iter()
        .map(|(conditions, criteria)| {
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (conditions, criteria),
            )
        })
        .sum::<u64>()
}

fn part2(input: &str) -> u64 {
    let (_, records) = records(input).unwrap();
    records
        .iter()
        .map(|(conditions, criteria)| {
            (
                [
                    &conditions[..],
                    &[SpringCondition::Unknown],
                    &conditions[..],
                    &[SpringCondition::Unknown],
                    &conditions[..],
                    &[SpringCondition::Unknown],
                    &conditions[..],
                    &[SpringCondition::Unknown],
                    &conditions[..],
                ]
                .concat(),
                [
                    &criteria[..],
                    &criteria[..],
                    &criteria[..],
                    &criteria[..],
                    &criteria[..],
                ]
                .concat(),
            )
        })
        .map(|(conditions, criteria)| {
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&conditions, &criteria),
            )
        })
        .sum::<u64>()
}

fn main() {
    let args = Cli::parse();
    let input = std::fs::read_to_string(args.path).unwrap();
    println!("Part 1: {}", part1(&input));
    println!("Part 2: {}", part2(&input));
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_record() {
        let input = "???.### 1,1,3";
        assert_eq!(
            record(input),
            Ok((
                "",
                (
                    vec![
                        SpringCondition::Unknown,
                        SpringCondition::Unknown,
                        SpringCondition::Unknown,
                        SpringCondition::Operational,
                        SpringCondition::Damaged,
                        SpringCondition::Damaged,
                        SpringCondition::Damaged,
                    ],
                    vec![1, 1, 3],
                )
            ))
        );
    }

    #[test]
    fn test_parse_records() {
        let input = "???.### 1,1,3\n.??..??...?##. 1,1,3\n?#?#?#?#?#?#?#? 1,3,1,6";
        assert_eq!(
            records(input),
            Ok((
                "",
                vec![
                    (
                        vec![
                            SpringCondition::Unknown,
                            SpringCondition::Unknown,
                            SpringCondition::Unknown,
                            SpringCondition::Operational,
                            SpringCondition::Damaged,
                            SpringCondition::Damaged,
                            SpringCondition::Damaged,
                        ],
                        vec![1, 1, 3],
                    ),
                    (
                        vec![
                            SpringCondition::Operational,
                            SpringCondition::Unknown,
                            SpringCondition::Unknown,
                            SpringCondition::Operational,
                            SpringCondition::Operational,
                            SpringCondition::Unknown,
                            SpringCondition::Unknown,
                            SpringCondition::Operational,
                            SpringCondition::Operational,
                            SpringCondition::Operational,
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Damaged,
                            SpringCondition::Operational,
                        ],
                        vec![1, 1, 3],
                    ),
                    (
                        vec![
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Unknown,
                            SpringCondition::Damaged,
                            SpringCondition::Unknown,
                        ],
                        vec![1, 3, 1, 6],
                    )
                ]
            ))
        );
    }

    #[test]
    fn test_known_combinations() {
        assert_eq!(
            valid_combos(&Bump::new(), &mut HashMap::default(), (&[], &[])),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&[SpringCondition::Damaged], &[1])
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&[SpringCondition::Damaged], &[2])
            ),
            0
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&[SpringCondition::Operational], &[])
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&[SpringCondition::Operational], &[1])
            ),
            0
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[SpringCondition::Operational, SpringCondition::Operational],
                    &[]
                )
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[SpringCondition::Operational, SpringCondition::Operational],
                    &[1]
                )
            ),
            0
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[SpringCondition::Operational, SpringCondition::Operational],
                    &[1]
                )
            ),
            0
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[SpringCondition::Operational, SpringCondition::Operational],
                    &[1, 1]
                )
            ),
            0
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[
                        SpringCondition::Operational,
                        SpringCondition::Operational,
                        SpringCondition::Operational
                    ],
                    &[]
                )
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[
                        SpringCondition::Operational,
                        SpringCondition::Operational,
                        SpringCondition::Operational
                    ],
                    &[1]
                )
            ),
            0
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[
                        SpringCondition::Operational,
                        SpringCondition::Damaged,
                        SpringCondition::Operational
                    ],
                    &[1]
                )
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (
                    &[
                        SpringCondition::Damaged,
                        SpringCondition::Operational,
                        SpringCondition::Damaged,
                    ],
                    &[1, 1]
                )
            ),
            1
        );
    }

    #[test]
    fn test_known_example() {
        let input = "#.#.### 1,1,3\n.#...#....###. 1,1,3\n.#.###.#.###### 1,3,1,6\n####.#...#... 4,1,1\n#....######..#####. 1,6,5\n.###.##....# 3,2,1";
        let (_, records) = records(input).unwrap();
        assert_eq!(records.len(), 6);
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[0].0, &records[0].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[1].0, &records[1].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[2].0, &records[2].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[3].0, &records[3].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[4].0, &records[4].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[5].0, &records[5].1)
            ),
            1
        );
    }

    #[test]
    fn test_example_unknown() {
        let input = "???.### 1,1,3\n.??..??...?##. 1,1,3\n?#?#?#?#?#?#?#? 1,3,1,6\n????.#...#... 4,1,1\n????.######..#####. 1,6,5\n?###???????? 3,2,1";
        let (_, records) = records(input).unwrap();
        assert_eq!(records.len(), 6);
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[0].0, &records[0].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[1].0, &records[1].1)
            ),
            4
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[2].0, &records[2].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[3].0, &records[3].1)
            ),
            1
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[4].0, &records[4].1)
            ),
            4
        );
        assert_eq!(
            valid_combos(
                &Bump::new(),
                &mut HashMap::default(),
                (&records[5].0, &records[5].1)
            ),
            10
        );
    }

    #[test]
    fn test_example_part1() {
        let input = "???.### 1,1,3\n.??..??...?##. 1,1,3\n?#?#?#?#?#?#?#? 1,3,1,6\n????.#...#... 4,1,1\n????.######..#####. 1,6,5\n?###???????? 3,2,1";
        assert_eq!(part1(input), 21);
    }

    #[test]
    fn test_example_part2() {
        let input = "???.### 1,1,3\n.??..??...?##. 1,1,3\n?#?#?#?#?#?#?#? 1,3,1,6\n????.#...#... 4,1,1\n????.######..#####. 1,6,5\n?###???????? 3,2,1";
        assert_eq!(part2(input), 525152);
    }
}
