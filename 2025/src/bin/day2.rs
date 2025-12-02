use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn is_mirror(n: i64) -> bool {
    let s = n.to_string();
    let length = s.len();
    if length % 2 != 0 {
        return false;
    }
    return s[..length / 2] == s[(length / 2)..];
}

fn part1(input: &str) -> i64 {
    input
        .split(',')
        .map(|range| range.split_once('-').unwrap())
        .flat_map(|(start, end)| start.parse::<i64>().unwrap()..=end.parse::<i64>().unwrap())
        .filter(|x| is_mirror(*x))
        .sum()
}

fn is_invalid(n: i64) -> bool {
    let s = n.to_string();
    let s = s.as_bytes();
    let length = s.len();
    'next: for j in 1..=length / 2 {
        if length % j != 0 {
            continue;
        }
        let times = length / j;
        for i in 0..j {
            let last = s[i];
            for k in 1..times {
                if last != s[k * j + i] {
                    continue 'next;
                }
            }
        }
        return true;
    }
    false
}

fn part2(input: &str) -> i64 {
    input
        .split(',')
        .map(|range| range.split_once('-').unwrap())
        .flat_map(|(start, end)| start.parse().unwrap()..=end.parse().unwrap())
        .filter(|x| is_invalid(*x))
        .sum()
}

fn main() {
    let cli = Cli::parse();

    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");

    let result_part1 = part1(&input);
    println!("Part 1 Result: {result_part1}");

    println!("Part 2 Result: {}", part2(&input));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror() {
        assert_eq!(is_mirror(123123), true);
        assert_eq!(is_mirror(45562), false);
        assert_eq!(is_mirror(99), true);
        assert_eq!(is_mirror(45445), false);
    }

    #[test]
    fn test_invalid() {
        assert_eq!(is_invalid(11), true);
        assert_eq!(is_invalid(123), false);
        assert_eq!(is_invalid(1212), true);
        assert_eq!(is_invalid(123123123), true);
    }
}
