use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn parse_stack(pic: &str) -> Vec<Vec<char>> {
    let re = Regex::new(r"(\[([A-Z])\]|\s\s\s)\s?").unwrap();
    let mut stacks = vec![];
    for (line_count, line) in pic.lines().rev().enumerate() {
        if line_count == 0 {
            continue
        }
        for (num, cap) in re.captures_iter(line).enumerate() {
            // handle starting initialisation
            if num >= stacks.len() {
                stacks.push(vec![]);
            }

            if let Some(pos_stack) = stacks.get_mut(num) {
                if let Some(letter) = cap.get(2) {
                    pos_stack.push(letter.as_str().chars().next().unwrap());
                }
            } else {
                panic!("Uninitialized position");
            }
        }
    }
    stacks
}

fn instruction_parser(instruction: &str) -> (usize,usize,usize) {
    let re = Regex::new(r"move (\d+) from (\d+) to (\d+)").unwrap();
    let cap = re.captures(instruction).unwrap();
    (
        cap.get(1).unwrap().as_str().parse::<usize>().unwrap(),
        cap.get(2).unwrap().as_str().parse::<usize>().unwrap() - 1, // handle start-from-zero
        cap.get(3).unwrap().as_str().parse::<usize>().unwrap() - 1, // handle start-from-zero
    )
}


#[allow(dead_code)]
fn perform_instructions<'a>(mut stacks: Vec<Vec<char>>, ins: &str) -> Vec<Vec<char>>{
    for line in ins.lines() {
        let (mv, from, to) = instruction_parser(line);
        for _ in 0..mv {
            let tmp = stacks.get_mut(from).unwrap().pop().unwrap();
            stacks.get_mut(to).unwrap().push(tmp);
        }
    }
    stacks
}

fn stack_mover<'a>(mut stacks: Vec<Vec<char>>, ins: &str) -> Vec<Vec<char>> {
    let mut mover = vec![];
    for line in ins.lines() {
        let (mv, from, to) = instruction_parser(line);
        for _ in 0..mv {
            let tmp = stacks.get_mut(from).unwrap().pop().unwrap();
            mover.push(tmp);
        }
        for _ in 0..mv {
            let tmp = mover.pop().unwrap();
            stacks.get_mut(to).unwrap().push(tmp);
        }
    }
    stacks
}

fn spell(stacks: Vec<Vec<char>>) -> String { 
    let mut result = String::new();
    for stack in stacks {
        let item = stack.last().unwrap();
        result.push(*item);
    }
    result
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let mut dev = input.split("\n\n");
    let stacks_input = dev.next().unwrap();
    let instruction_input = dev.next().unwrap();

    println!("{:?}", parse_stack(stacks_input));
    let end = stack_mover(parse_stack(stacks_input), instruction_input);
    println!("{:?}", end);
    let s = spell(end);
    println!("{:?}", s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
    }
}
