use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(PartialEq, Clone, Debug)]
struct Board {
    entries: Vec<Cell>,
}
#[derive(PartialEq, Clone, Debug)]
struct Cell {
    value: usize,
    mark: bool,
}

impl Board {
    fn height(&self) -> usize {
        5
    }
    fn width(&self) -> usize {
        5
    }
}

fn is_completed(board: &Board) -> bool {
    let mut count = 0;
    // horizontal
    for y in 0..board.width() {
        count = 0;
        for x in 0..board.height() {
            let mark =  board.entries.get(y*board.width() + x).expect("out of bounds").mark;
            count = count + match mark {
                true => 1,
                false => 0,
            }
        }
        if count == 5 {
            return true
        }

    }
    for x in 0..board.width() {
        count = 0;
        for y in 0..board.height() {
            let mark =  board.entries.get(y*board.width() + x).expect("out of bounds").mark;
            count = count + match mark {
                true => 1,
                false => 0,
            }
        }
        if count == 5 {
            return true
        }
    }
    false
}

fn fill_in(inputs: Vec<usize>, mut boards: Vec<Board>) -> Option<(usize, Board)> {
    for input in inputs.iter() {
        for board in &mut boards {
            if let Some(index) = board.entries.iter().position(|n| &n.value == input) {
                let mark = &mut board.entries.get_mut(index).unwrap().mark;
                *mark = true;
            }
            if is_completed(board) {
                return Some((input.clone(), board.clone()));
            }
        }
    }
    None
}

fn parse_bingo(source: &str) -> (Vec<usize>, Vec<Board>) {
    let mut boards = Vec::new();
    let mut board;
    let mut next;
    let inputs;
    let (a, mut b, mut c);
    a = parse_inputs(source);
    inputs = a.0;
    next = a.1;
    while next.len() > 3 {
        b = parse_double_newlines(next);
        match b.0 {
            Some(()) => (),
            None => break,
        }
        next = b.1;
        c = parse_bingo_board(next);
        board = c.0;
        next = c.1;
        boards.push(board);
    }
    (inputs, boards)
}

fn parse_inputs(source: &str) -> (Vec<usize>, &str) {
    let end_index = &source.find('\n').unwrap();
    let (text_inputs, remaining_input) = source.split_at(*end_index);
    let inputs = text_inputs
        .split(',')
        .map(|x| x.parse::<usize>().unwrap())
        .collect::<Vec<usize>>();
    (inputs, remaining_input)
}

fn parse_double_newlines(source: &str) -> (Option<()>, &str) {
    if source.chars().nth(0).unwrap() != '\n' || source.chars().nth(1).unwrap() != '\n' {
        (None, &source)
    } else {
        (Some(()), &source[2..])
    }
}

fn parse_bingo_board(source: &str) -> (Board, &str) {
    let newline_regex = Regex::new("\n\n").unwrap();
    let space_regex = Regex::new(" +").unwrap();
    let split_index = newline_regex
        .find(source)
        .expect("Could not find double new line")
        .start();
    let (text_board, remaining_input) = source.split_at(split_index);
    let board_entries = text_board
        .split('\n')
        .map(|row| row.trim())
        .map(|row| space_regex.split(row))
        .flatten()
        .map(|num| Cell {
            value: num.parse::<usize>().unwrap(),
            mark: false,
        })
        .collect::<Vec<Cell>>();

    (
        Board {
            entries: board_entries,
        },
        &remaining_input
    )
}

fn main() {
    let args = Cli::from_args();
    let mut source = fs::read_to_string(args.path.as_path()).unwrap();
    //buf_reader.read_to_string(&mut source);
    let (inputs, boards) = parse_bingo(source.as_ref());
    if let Some((last_call, filled_board)) = fill_in(inputs, boards){
        println!("{:?}", &filled_board);
        let score = filled_board.entries.iter().filter(|e| e.mark == false).map(|e| e.value).fold(0, |a, e| a + e) * last_call;
        println!("Score: {}", score);
    } else {
        println!("No winners found");
    }
}
