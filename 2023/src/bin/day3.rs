use std::{collections::HashMap, fs::read, time::Instant};

use clap::{command, Parser};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Tile {
    Period,
    Symbol(char),
    Number(u32),
}

impl From<char> for Tile {
    fn from(c: char) -> Self {
        match c {
            '.' => Tile::Period,
            number if number.is_ascii_digit() => Tile::Number(c.to_digit(10).unwrap()),
            c => Tile::Symbol(c),
        }
    }
}

struct Schematic {
    tiles: Vec<Tile>,
    width: i32,
    height: i32,
}

impl Schematic {
    fn new(input: &str) -> Option<Self> {
        Some(Self {
            tiles: input
                .chars()
                .filter(|c| *c != '\n')
                .map(Tile::from)
                .collect(),
            width: input.find('\n').unwrap() as i32,
            height: input.lines().count() as i32,
        })
    }

    fn get(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return Tile::Period;
        }
        self.tiles[(y * self.width + x) as usize]
    }
}

fn find_number_coords(schematic: &Schematic) -> Vec<((i32, i32), i32)> {
    let mut number_coords = vec![];
    for y in 0..schematic.height {
        let mut current = None;
        for x in 0..schematic.width {
            match schematic.get(x, y) {
                Tile::Number(_) => {
                    if let Some((start, _)) = current {
                        current = Some((start, x));
                    } else {
                        current = Some(((x, y), x));
                    }
                }
                _ => {
                    if let Some(number) = current {
                        number_coords.push(number);
                        current = None;
                    }
                }
            }
        }
        if let Some(number) = current {
            number_coords.push(number);
        }
    }
    number_coords
}

fn enumerate_surrounding_coords(number_coord: ((i32, i32), i32)) -> Vec<(i32, i32)> {
    let ((start_x, y), end_x) = number_coord;
    let mut coords = Vec::with_capacity(6 + (end_x - start_x) as usize * 2);
    // enumerate start
    coords.push((start_x - 1, y - 1));
    coords.push((start_x - 1, y));
    coords.push((start_x - 1, y + 1));

    // enumerate end
    coords.push((end_x + 1, y - 1));
    coords.push((end_x + 1, y));
    coords.push((end_x + 1, y + 1));

    // enumerate above and below
    for x in start_x..=end_x {
        coords.push((x, y - 1));
        coords.push((x, y + 1));
    }

    coords
}

fn get_number(schematic: &Schematic, number_coord: ((i32, i32), i32)) -> u32 {
    let (start_x, y) = (number_coord.0 .0, number_coord.0 .1);
    let end_x = number_coord.1;
    (start_x..=end_x)
        .map(|x| match schematic.get(x, y) {
            Tile::Number(v) => v,
            v => panic!("found not a number: {:?}", (x, y, v)),
        })
        .fold(0, |acc, i| acc * 10 + i) // convert digits to number
}

fn sum_of_part_numbers(schematic: &Schematic) -> u32 {
    find_number_coords(schematic)
        .into_iter()
        .map(|number| (number, enumerate_surrounding_coords(number)))
        .filter_map(|(number_coord, surrounding)| {
            surrounding
                .iter()
                .any(|(x, y)| matches!(schematic.get(*x, *y), Tile::Symbol(_)))
                .then(|| get_number(schematic, number_coord))
        })
        .sum()
}

fn get_gear_ratio(schematic: &Schematic) -> u32 {
    find_number_coords(schematic)
        .into_iter()
        .map(|number| (number, enumerate_surrounding_coords(number)))
        .flat_map(|(number_coord, surrounding)| {
            surrounding
                .into_iter()
                .filter_map(move |(x, y)| match schematic.get(x, y) {
                    Tile::Symbol('*') => Some((number_coord, (x, y))),
                    _ => None,
                })
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<(i32, i32), Vec<u32>>, (number_coord, gear)| {
                acc.entry(gear)
                    .or_default()
                    .push(get_number(schematic, number_coord));
                acc
            },
        )
        .into_values()
        .filter_map(|number| {
            if number.len() == 2 {
                Some(number[0] * number[1])
            } else {
                None
            }
        })
        .sum()
}

fn main() {
    let args = Cli::parse();
    let start = Instant::now();
    let input = read(args.path.as_path()).unwrap();
    let schematic = Schematic::new(std::str::from_utf8(&input).unwrap()).unwrap();
    //println!("Part 1: {}", sum_of_part_numbers(&schematic));
    println!("Part 2: {}", get_gear_ratio(&schematic));
    println!("Time elapsed: {:?}", start.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schematic() {
        let input = include_str!("../../input/day3-example");
        let schematic = Schematic::new(input).unwrap();
        assert_eq!(schematic.get(0, 0), Tile::Number(4));
        assert_eq!(schematic.get(1, 0), Tile::Number(6));
        assert_eq!(schematic.get(-1, 0), Tile::Period);
        assert_eq!(schematic.get(0, 1), Tile::Period);
        assert_eq!(schematic.get(3, 1), Tile::Symbol('*'));
    }

    #[test]
    fn test_example() {
        let input = include_str!("../../input/day3-example");
        let schematic = Schematic::new(input).unwrap();
        assert_eq!(sum_of_part_numbers(&schematic), 4361);
    }

    #[test]
    fn test_get_number() {
        let input = include_str!("../../input/day3-example");
        let schematic = Schematic::new(input).unwrap();
        let number_coord = ((2, 2), 3);
        assert_eq!(get_number(&schematic, number_coord), 35);
    }

    #[test]
    fn test_enumerate_surrounding_coords() {
        let number_coord = ((5, 9), 7);
        let coords = enumerate_surrounding_coords(number_coord);
        println!("{:?}", coords);
        assert_eq!(coords.len(), 12);
        assert!(coords.contains(&(4, 8)));
        assert!(coords.contains(&(4, 9)));
        assert!(coords.contains(&(4, 10)));
        assert!(coords.contains(&(8, 8)));
        assert!(coords.contains(&(8, 9)));
        assert!(coords.contains(&(8, 10)));
        assert!(coords.contains(&(5, 8)));
        assert!(coords.contains(&(5, 10)));
        assert!(coords.contains(&(6, 8)));
        assert!(coords.contains(&(6, 10)));
        assert!(coords.contains(&(7, 8)));
        assert!(coords.contains(&(7, 10)));
    }

    #[test]
    fn test_real_world() {
        let input = include_str!("../../input/day3-single-digit-example");
        let schematic = Schematic::new(input).unwrap();
        assert_eq!(sum_of_part_numbers(&schematic), 65);
    }

    #[test]
    fn test_example_get_gear_ratio() {
        let input = include_str!("../../input/day3-example");
        let schematic = Schematic::new(input).unwrap();
        assert_eq!(get_gear_ratio(&schematic), 467835);
    }
}
