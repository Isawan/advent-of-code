use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Moved(isize),
    Unmoved(isize),
}

impl State {
    fn value(&self) -> isize {
        match self {
            State::Moved(i) => *i,
            State::Unmoved(i) => *i,
        }
    }
}
impl From<&isize> for State {
    fn from(i: &isize) -> Self {
        State::Unmoved(*i)
    }
}
impl From<isize> for State {
    fn from(i: isize) -> Self {
        State::Unmoved(i)
    }
}

fn parse(input: &str) -> Vec<isize> {
    input.lines().map(|line| line.parse().unwrap()).collect()
}

fn move_right(file: &mut Vec<impl Clone + std::fmt::Debug>, from: usize, amount: usize) {
    let size = file.len();
    let to = from + amount;
    let hold = file.remove(from);
    let new_index = to.rem_euclid(size - 1);
    file.insert(new_index, hold);
}

fn move_left(file: &mut Vec<impl Clone + std::fmt::Debug>, from: usize, amount: usize) {
    let size = file.len() as isize;
    let to = (from as isize) - (amount as isize);
    let hold = file.remove(from);
    let new_index = to.rem_euclid(size - 1);
    if new_index == 0 {
        file.push(hold);
    } else {
        file.insert(new_index as usize, hold);
    }
}

fn move_once(file: &mut Vec<State>) -> Option<()> {
    let search_unmoved = file.iter().position(|c| match c {
        State::Moved(_) => false,
        State::Unmoved(_) => true,
    });
    let size = file.len() as isize;
    if let Some(index) = search_unmoved {
        let value = file[index].value();
        file[index] = State::Moved(value);
        if value > 0 {
            move_right(file, index, value.abs() as usize);
        } else if value == 0 {
            // do nothing
        } else {
            move_left(file, index, value.abs() as usize);
        }
        return Some(());
    } else {
        return None;
    }
}

fn mix(mut input: Vec<isize>) -> Vec<isize> {
    let mut file: Vec<State> = input.drain(..).map(State::from).collect();
    let mut count = 0;
    while let Some(_) = move_once(&mut file) {
        count += 1;
    }
    input.extend(file.iter().map(State::value));
    input
}

fn sum_of_coordinates(input: &Vec<isize>) -> isize {
    let zero_position = input.iter().position(|x| x == &0).unwrap();
    input[(zero_position + 1000usize).rem_euclid(input.len())]
        + input[(zero_position + 2000usize).rem_euclid(input.len())]
        + input[(zero_position + 3000usize).rem_euclid(input.len())]
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    println!("solution 1: {}", sum_of_coordinates(&mix(parse(&input))));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_right() {
        let mut input = vec![1, 2, -3, 3, -2, 0, 4];
        move_right(&mut input, 0, 1);
        assert_eq!(input, vec![2, 1, -3, 3, -2, 0, 4]);
        move_right(&mut input, 0, 2);
        assert_eq!(input, vec![1, -3, 2, 3, -2, 0, 4]);

        let mut input = vec![1, 2, 3, -2, -3, 0, 4];
        move_right(&mut input, 2, 3);
        assert_eq!(input, vec![1, 2, -2, -3, 0, 3, 4]);

        let mut input = vec![1, 2, -3, 0, 3, 4, -2];
        move_right(&mut input, 5, 4);
        assert_eq!(input, vec![1, 2, -3, 4, 0, 3, -2]);
    }

    #[test]
    fn test_move_far_right() {
        let mut input = vec![0, 1, 2, 3, 4, 5, 6];
        move_right(&mut input, 5, 8);
        assert_eq!(input, vec![0, 5, 1, 2, 3, 4, 6]);
    }

    #[test]
    fn test_move_far_left() {
        let mut input = vec![0, 1, 2, 3, 4, 5, 6];
        move_left(&mut input, 5, 8);
        assert_eq!(input, vec![0, 1, 2, 5, 3, 4, 6]);
    }

    #[test]
    fn test_move_left() {
        let mut input = vec![1, -3, 2, 3, -2, 0, 4];
        move_left(&mut input, 1, 3);
        assert_eq!(input, vec![1, 2, 3, -2, -3, 0, 4]);

        let mut input = vec![1, 2, -2, -3, 0, 3, 4];
        move_left(&mut input, 2, 2);
        assert_eq!(input, vec![1, 2, -3, 0, 3, 4, -2]);

        let mut input = vec![1, 2, -2, -3, 0, 3, 4];
        move_left(&mut input, 2, 1);
        assert_eq!(input, vec![1, -2, 2, -3, 0, 3, 4]);
    }

    #[test]
    fn test_move_once() {
        let mut input: Vec<State> = parse(include_str!("../../input/day20-test"))
            .iter()
            .map(State::from)
            .collect();
        move_once(&mut input);
        assert_eq!(
            input,
            vec![
                State::Unmoved(2),
                State::Moved(1),
                State::Unmoved(-3),
                State::Unmoved(3),
                State::Unmoved(-2),
                State::Unmoved(0),
                State::Unmoved(4)
            ]
        );
        move_once(&mut input);
        assert_eq!(
            input,
            vec![
                State::Moved(1),
                State::Unmoved(-3),
                State::Moved(2),
                State::Unmoved(3),
                State::Unmoved(-2),
                State::Unmoved(0),
                State::Unmoved(4)
            ]
        );
    }

    #[test]
    fn test_mix() {
        let input = parse(include_str!("../../input/day20-test"));
        let output = mix(input);
        assert_eq!(output, vec![1, 2, -3, 4, 0, 3, -2]);
    }

    #[test]
    fn test_coords() {
        let input = parse(include_str!("../../input/day20-test"));
        let output = mix(input);
        assert_eq!(sum_of_coordinates(&output), 3);
    }
}
