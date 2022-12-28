use std::{collections::VecDeque, time::Instant};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn snafu_to_number(snafu: &str) -> i64 {
    let mut accum = 0;
    for (i, c) in snafu.chars().rev().enumerate() {
        accum += 5_i64.pow(i as u32)
            * match c {
                '=' => -2,
                '-' => -1,
                '0' => 0,
                '1' => 1,
                '2' => 2,
                _ => panic!("unexpected character"),
            }
    }
    accum
}

fn get_snafu_digits(number: i64) -> u32 {
    for i in 1.. {
        let power = 2 * 5_u32.pow(i);
        if power >= number as u32 {
            return i;
        }
    }
    unreachable!()
}
fn number_to_snafu(mut number: i64) -> String {
    let mut digit_string = VecDeque::new();
    loop {
        println!("loop");
        let mut rem = number % 5;
        number = number / 5;
        // generalized from the balanced ternary number system
        if rem == 3 {
            rem = -2;
            number += 1;
        }
        if rem == 4 {
            rem = -1;
            number += 1;
        }
        match rem {
            -2 => digit_string.push_front('='),
            -1 => digit_string.push_front('-'),
            0 => digit_string.push_front('0'),
            1 => digit_string.push_front('1'),
            2 => digit_string.push_front('2'),
            _ => unreachable!("unexpected"),
        }
        if number == 0 {
            break;
        }
    }

    //println!("{}", digit_string);
    //println!("---");
    digit_string
        .into_iter()
        .collect::<String>()
        .trim_start_matches('0')
        .to_string()
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    let result = input.lines().fold(0, |a, line| a + snafu_to_number(line));
    println!("solution 1: {}", number_to_snafu(result));
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snafu_to_number() {
        assert_eq!(snafu_to_number("1=-0-2"), 1747);
        assert_eq!(snafu_to_number("12111"), 906);
        assert_eq!(snafu_to_number("2=0="), 198);
        assert_eq!(snafu_to_number("21"), 11);
        assert_eq!(snafu_to_number("2=01"), 201);
        assert_eq!(snafu_to_number("111"), 31);
        assert_eq!(snafu_to_number("20012"), 1257);
        assert_eq!(snafu_to_number("112"), 32);
        assert_eq!(snafu_to_number("1=-1="), 353);
        assert_eq!(snafu_to_number("1-12"), 107);
        assert_eq!(snafu_to_number("12"), 7);
        assert_eq!(snafu_to_number("1="), 3);
        assert_eq!(snafu_to_number("122"), 37);
    }

    #[test]
    fn test_number_to_snafu() {
        assert_eq!(number_to_snafu(1).as_str(), "1");
        assert_eq!(number_to_snafu(2).as_str(), "2");
        assert_eq!(number_to_snafu(3).as_str(), "1=");
        assert_eq!(number_to_snafu(4).as_str(), "1-");
        assert_eq!(number_to_snafu(5).as_str(), "10");
        assert_eq!(number_to_snafu(6).as_str(), "11");
        assert_eq!(number_to_snafu(7).as_str(), "12");
        assert_eq!(number_to_snafu(8).as_str(), "2=");
        assert_eq!(number_to_snafu(9).as_str(), "2-");
        assert_eq!(number_to_snafu(10).as_str(), "20");
        assert_eq!(number_to_snafu(15).as_str(), "1=0");
        assert_eq!(number_to_snafu(20).as_str(), "1-0");
        assert_eq!(number_to_snafu(2022).as_str(), "1=11-2");
        assert_eq!(number_to_snafu(12345).as_str(), "1-0---0");
        assert_eq!(number_to_snafu(314159265).as_str(), "1121-1110-1=0");
    }
}
