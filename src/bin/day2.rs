use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}
enum Direction {
    Forward,
    Up,
    Down,
}

fn parse_line(line: &str) -> Option<(Direction, i32)> {
    let mut tokens = line.split(' ');
    let direction_str = tokens.next()?;
    let direction = match direction_str {
        "up" => Direction::Up,
        "down" => Direction::Down,
        "forward" => Direction::Forward,
        _ => return None
    };
    if let Ok(count) = tokens.next()?.parse::<i32>() {
        // handle the case where more than two tokens a present
        match tokens.next() {
            Some(_) => None,
            None => Some((direction, count)),
        }
    } else {
        return None;
    }
}

fn final_position(movement_stream: impl BufRead) -> (i32, i32) {
    let mut horizontal_position = 0;
    let mut depth = 0;
    let mut aim = 0;
    let lines = movement_stream.lines();
    for line in lines {
        println!("x={} y={}", horizontal_position, depth);
        let next_line = line
            .as_ref()
            .expect("Could not read next line due to IO error");
        if let Some((direction, count)) = parse_line(next_line) {
            aim = aim
                + match direction {
                    Direction::Up => -count,
                    Direction::Down => count,
                    Direction::Forward => 0,
                };
            horizontal_position = horizontal_position
                + match direction {
                    Direction::Up => 0,
                    Direction::Down => 0,
                    Direction::Forward => count,
                };
            depth = depth + match direction {
                    Direction::Up => 0,
                    Direction::Down => 0,
                    Direction::Forward => aim * count,
            };
        } else {
            break;
        }
    }
    return (horizontal_position, depth)
}

fn main() {
    let args = Cli::from_args();
    let file = File::open(args.path.as_path()).unwrap();
    let buf_reader = BufReader::new(file);
    let (x,y) = final_position(buf_reader);
    println!("x: {} y: {}", x, y);
    println!("{}", x * y);
}
