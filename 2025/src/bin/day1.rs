use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Direction {
    R,
    L,
}

fn parse(line: &str) -> Option<(Direction, i32)> {
    line.split_at_checked(1).map(|(c, i)| {
        let direction = match c {
            "R" => Direction::R,
            "L" => Direction::L,
            _ => panic!("err"),
        };
        (direction, i.parse().expect("not an int"))
    })
}

fn part1(input: &str) -> usize {
    input
        .lines()
        .map(|x| parse(x).unwrap())
        .scan(50, |a, (d, i)| {
            *a = match d {
                Direction::R => (*a + i).rem_euclid(100),
                Direction::L => (*a - i).rem_euclid(100),
            };
            Some(*a)
        })
        .filter(|x| *x == 0)
        .count()
}

fn part2(input: &str) -> usize {
    input
        .lines()
        .map(|x| parse(x).unwrap())
        .fold((50, 0), |(a, c), (d, i)| {
            let b = match d {
                Direction::R => (a + i).rem_euclid(100),
                Direction::L => (a - i).rem_euclid(100),
            };
            let q = match d {
                Direction::R => (a + i).div_euclid(100),
                Direction::L if a - i > 0 => 0,
                Direction::L if a - i == 0 => 1,
                Direction::L if a == 0 => ((a - i) / (-100)),
                Direction::L => ((a - i) / (-100)) + 1,
            };
            (b, c + q)
        })
        .1 as usize
}

fn main() {
    let cli = Cli::parse();

    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");

    let result_part1 = part1(&input);
    println!("Part 1 Result: {}", result_part1);


    let result_part2 = part2(&input);
    println!("Part 2 Result: {}", result_part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(parse("R50"), Some((Direction::R, 50)))
    }
}
