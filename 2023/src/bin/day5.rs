use std::{fs::read, time::Instant};

use clap::Parser;
use nom::{
    bytes::complete::{is_a, is_not, tag},
    character::complete::{digit1, line_ending, multispace0, multispace1, newline},
    combinator::{map_res, opt},
    multi::{self, many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

type Instruction = (i64, i64, i64);
type Almanac<'a> = (Vec<i64>, Vec<(&'a str, Vec<Instruction>)>);

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn number(input: &str) -> IResult<&str, i64> {
    map_res(delimited(multispace0, digit1, multispace0), str::parse)(input)
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    tuple((number, number, number))(input)
}

fn instructions(input: &str) -> IResult<&str, Vec<Instruction>> {
    many1(terminated(instruction, many0(line_ending)))(input)
}

fn map(input: &str) -> IResult<&str, (&str, Vec<Instruction>)> {
    tuple((terminated(is_not(" "), tag(" map:\n")), instructions))(input)
}

fn maps(input: &str) -> IResult<&str, Vec<(&str, Vec<Instruction>)>> {
    multi::many1(terminated(map, many0(line_ending)))(input)
}

fn seeds(input: &str) -> IResult<&str, Vec<i64>> {
    delimited(tag("seeds:"), many1(number), many0(newline))(input)
}

fn almanac(input: &str) -> IResult<&str, (Vec<i64>, Vec<(&str, Vec<Instruction>)>)> {
    pair(seeds, maps)(input)
}

fn transform(seed: i64, &(destination, source, range): &Instruction) -> Option<i64> {
    let shift = destination - source;
    if seed >= source && seed < source + range {
        Some(seed + shift)
    } else {
        None
    }
}

fn transforms(seed: i64, instructions: &[Instruction]) -> i64 {
    instructions
        .iter()
        .find_map(|ins| transform(seed, ins))
        .unwrap_or(seed)
}

fn apply_all_transforms<'a>(
    seed: i64,
    list_instructions: &Vec<(&'a str, Vec<Instruction>)>,
) -> i64 {
    list_instructions
        .iter()
        .fold(seed, |s, (_, ins)| transforms(s, ins))
}

fn part1((seeds, maps): &Almanac) -> i64 {
    seeds
        .into_iter()
        .map(|original_seed| apply_all_transforms(*original_seed, maps))
        .fold(i64::MAX, i64::min)
}

fn part2((seed_pairs_seq, maps): &Almanac) -> i64 {
    let seed_pairs: Vec<(i64, i64)> = seed_pairs_seq
        .chunks(2)
        .map(|pair| match pair {
            [a, b] => (*a, *b),
            _ => panic!("seed not paired"),
        })
        .collect();

    seed_pairs
        .par_iter()
        .flat_map(|(a, b)| (*a..*a + *b))
        .map(|original_seed| apply_all_transforms(original_seed, maps))
        .reduce(|| i64::MAX, i64::min)
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let f = read(args.path.as_path()).unwrap();
    let input = std::str::from_utf8(&f).unwrap();
    let input = almanac(input).unwrap().1;
    println!("Part 1: {}", part1(&input));
    println!("Part 2: {}", part2(&input));
    println!("Time elapsed: {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_map() {
        let input = "seed-to-soil map:
                   50 98 2
                   52 50 48";
        assert!(map(input).is_ok());
    }

    #[test]
    fn test_parse_maps() {
        let input = "seed-to-soil map:
                    50 98 2
                    52 50 48
                     
                    soil-to-fertilizer map:
                    0 15 37
                    37 52 2
                    39 0 15
                    
                    fertilizer-to-water map:
                    49 53 8
                    0 11 42
                    42 0 7
                    57 7 4";
        let result = maps(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_seeds() {
        let input = "seeds: 79 14 55 13";
        let result = seeds(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_example() {
        let input = "seeds: 79 14 55 13

                     seed-to-soil map:
                     50 98 2
                     52 50 48
                     
                     soil-to-fertilizer map:
                     0 15 37
                     37 52 2
                     39 0 15
                     
                     fertilizer-to-water map:
                     49 53 8
                     0 11 42
                     42 0 7
                     57 7 4
                     
                     water-to-light map:
                     88 18 7
                     18 25 70
                     
                     light-to-temperature map:
                     45 77 23
                     81 45 19
                     68 64 13
                     
                     temperature-to-humidity map:
                     0 69 1
                     1 0 69
                     
                     humidity-to-location map:
                     60 56 37
                     56 93 4";

        let expected = (
            vec![79, 14, 55, 13],
            vec![
                ("seed-to-soil", vec![(50, 98, 2), (52, 50, 48)]),
                (
                    "soil-to-fertilizer",
                    vec![(0, 15, 37), (37, 52, 2), (39, 0, 15)],
                ),
                (
                    "fertilizer-to-water",
                    vec![(49, 53, 8), (0, 11, 42), (42, 0, 7), (57, 7, 4)],
                ),
                ("water-to-light", vec![(88, 18, 7), (18, 25, 70)]),
                (
                    "light-to-temperature",
                    vec![(45, 77, 23), (81, 45, 19), (68, 64, 13)],
                ),
                ("temperature-to-humidity", vec![(0, 69, 1), (1, 0, 69)]),
                ("humidity-to-location", vec![(60, 56, 37), (56, 93, 4)]),
            ],
        );
        let result = almanac(input);
        assert_eq!(result.unwrap().1, expected);
    }

    #[test]
    fn test_transform() {
        let seed = 79;
        let ins = (52, 50, 48);
        assert_eq!(transform(seed, &ins), Some(81));
    }

    #[test]
    fn test_transforms() {
        let seeds = [79, 14, 55, 13];
        let maps = vec![(50, 98, 2), (52, 50, 48)];
        let soil: Vec<i64> = seeds.iter().map(|seed| transforms(*seed, &maps)).collect();
        assert_eq!(soil, vec![81, 14, 57, 13]);
    }

    #[test]
    fn test_example() {
        let (_, input) = almanac(
            "seeds: 79 14 55 13
    
                     seed-to-soil map:
                     50 98 2
                     52 50 48
    
                     soil-to-fertilizer map:
                     0 15 37
                     37 52 2
                     39 0 15
    
                     fertilizer-to-water map:
                     49 53 8
                     0 11 42
                     42 0 7
                     57 7 4
    
                     water-to-light map:
                     88 18 7
                     18 25 70
    
                     light-to-temperature map:
                     45 77 23
                     81 45 19
                     68 64 13
    
                     temperature-to-humidity map:
                     0 69 1
                     1 0 69
    
                     humidity-to-location map:
                     60 56 37
                     56 93 4",
        )
        .unwrap();
        let result = part1(&input);
        assert_eq!(result, 35);
        assert_eq!(part2(&input), 46);
    }

    // test all the individual steps separately
    #[test]
    fn test_seed_to_soil() {
        let maps = vec![(50, 98, 2), (52, 50, 48)];
        assert_eq!(transforms(79, &maps), 81);
        assert_eq!(transforms(14, &maps), 14);
        assert_eq!(transforms(55, &maps), 57);
        assert_eq!(transforms(13, &maps), 13);
    }

    #[test]
    fn test_soil_to_fertilizer() {
        let maps = vec![(0, 15, 37), (37, 52, 2), (39, 0, 15)];
        assert_eq!(transforms(81, &maps), 81);
        assert_eq!(transforms(14, &maps), 53);
        assert_eq!(transforms(57, &maps), 57);
        assert_eq!(transforms(13, &maps), 52);
    }

    #[test]
    fn test_fertilizer_to_water() {
        let maps = vec![(49, 53, 8), (0, 11, 42), (42, 0, 7), (57, 7, 4)];
        assert_eq!(transforms(81, &maps), 81);
        assert_eq!(transforms(53, &maps), 49);
        assert_eq!(transforms(57, &maps), 53);
        assert_eq!(transforms(52, &maps), 41);
    }

    #[test]
    fn test_water_to_light() {
        let maps = vec![(88, 18, 7), (18, 25, 70)];
        assert_eq!(transforms(81, &maps), 74);
        assert_eq!(transforms(49, &maps), 42);
        assert_eq!(transforms(53, &maps), 46);
        assert_eq!(transforms(41, &maps), 34);
    }

    #[test]
    fn test_light_to_temperature() {
        let maps = vec![(45, 77, 23), (81, 45, 19), (68, 64, 13)];
        assert_eq!(transforms(74, &maps), 78);
        assert_eq!(transforms(42, &maps), 42);
        assert_eq!(transforms(46, &maps), 82);
        assert_eq!(transforms(34, &maps), 34);
    }

    #[test]
    fn test_temperature_to_humidity() {
        let maps = vec![(0, 69, 1), (1, 0, 69)];
        assert_eq!(transforms(78, &maps), 78);
        assert_eq!(transforms(42, &maps), 43);
        assert_eq!(transforms(82, &maps), 82);
        assert_eq!(transforms(34, &maps), 35);
    }

    #[test]
    fn test_humidity_to_location() {
        let maps = vec![(60, 56, 37), (56, 93, 4)];
        assert_eq!(transforms(78, &maps), 82);
        assert_eq!(transforms(43, &maps), 43);
        assert_eq!(transforms(82, &maps), 86);
        assert_eq!(transforms(35, &maps), 35);
    }

    #[test]
    fn test_individual_transform_operator() {
        assert_eq!(transform(53, &(49, 53, 8)), Some(49));
        assert_eq!(transform(53, &(0, 11, 42)), None);
        assert_eq!(transform(53, &(42, 0, 7)), None);
        assert_eq!(transform(53, &(57, 7, 4)), None);
    }
}
