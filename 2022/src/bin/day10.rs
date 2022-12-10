use std::time::Instant;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct State {
    cycle: u32,
    register: i32,
    processing: u32,
    addition: i32,
}

impl State {
    fn new() -> Self {
        State {
            cycle: 0,
            register: 1,
            processing: 0,
            addition: 0,
        }
    }
    fn next(self, apply: (i32, u32)) -> Self {
        let (addition, cycle) = apply;
        State {
            cycle: self.cycle,
            register: self.register,
            processing: cycle,
            addition: addition,
        }
    }
    fn process(self) -> Self {
        State {
            cycle: self.cycle,
            register: self.register + self.addition,
            processing: 0,
            addition: 0,
        }
    }
    fn tick(self) -> Self {
        State {
            cycle: self.cycle + 1,
            register: self.register,
            processing: self.processing - 1,
            addition: self.addition,
        }
    }
}

fn parse_line(line: &str) -> (i32, u32) {
    let parts = line.split(' ').collect::<Vec<&str>>();
    match parts.as_slice() {
        ["noop"] => (0, 1),
        ["addx", addition] => (addition.parse::<i32>().unwrap(), 2),
        _ => panic!("unexpected"),
    }
}

fn run(mut state: State, instructions: &str) -> i32 {
    let mut sum = 0;
    for line in instructions.lines() {
        let ins = parse_line(line);
        state = state.next(ins);
        while state.processing != 0 {
            state = state.tick();
            sum = sum
                + if state.cycle >= 20 && (state.cycle - 20) % 40 == 0 {
                    (state.cycle as i32) * state.register
                } else {
                    0
                };
            if state.processing == 0 {
                state = state.process();
            }
        }
    }
    sum
}

fn draw(mut state: State, instructions: &str) -> Vec<Vec<u8>> {
    let mut crt = Vec::new();
    let mut row = vec![0; 40];
    for line in instructions.lines() {
        let ins = parse_line(line);
        state = state.next(ins);
        while state.processing != 0 {
            state = state.tick();
            let pos = (state.cycle as i32 - 1) % 40;
            if pos == state.register || pos - 1 == state.register || pos + 1 == state.register {
                row[pos as usize] = 1;
            }
            if state.cycle % 40 == 0 {
                crt.push(row);
                row = vec![0; 40];
            }
            if state.processing == 0 {
                state = state.process();
            }
        }
    }
    print!("\n");
    print!("\n");
    for row in crt.iter() {
        for c in row.iter() {
            print!("{}", if c == &1 { "1" } else { " " });
        }
        print!("\n");
    }
    print!("\n");
    print!("\n");
    crt
}

fn main() {
    let start = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    println!("sum: {}", run(State::new(), &input));

    draw(State::new(), &input);
    println!("time: {}", start.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_line() {
        assert_eq!(parse_line("noop"), (0, 1));
        assert_eq!(parse_line("addx 5"), (5, 2));
        assert_eq!(parse_line("addx -5"), (-5, 2));
    }

    #[test]
    fn test_run() {
        assert_eq!(
            run(State::new(), include_str!("../../input/day10-test")),
            13140
        );
    }
}
