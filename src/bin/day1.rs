use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::collections::VecDeque;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn count_increments_with_sliding_window(reader: impl BufRead) -> Result<i32, std::io::Error> {
    let window_size = 3;
    let mut increments = 0;
    let mut last_number: i32;
    let mut window = VecDeque::new();
    let mut lines = reader.lines();
    for line in &mut lines {
        let next_number = line.as_ref().expect("Could not read next line due to IO error")
            .parse::<i32>()
            .expect("Could not parse number");
        window.push_front(next_number);
        if window.len() == window_size { break; }
    }
    for line in lines {
        let next_line = line.as_ref().expect("Could not read next line due to IO error");
        if next_line == "" {
            // handle last line
            break;
        }
        let next_number = next_line
            .parse::<i32>()
            .expect("Could not parse number");
        last_number = window.iter().fold(0, |a, x| {a + x});
        window.push_front(next_number);
        let _ = window.pop_back();
        let window_sum = window.iter().fold(0, |a, x| {a + x});
        if window_sum > last_number {
            increments += 1;
        }
        println!("last : {}", last_number);
        println!("window : {}", window_sum);
    }
    Ok(increments)
}
fn main() {
    let args = Cli::from_args();
    let file = File::open(args.path.as_path()).unwrap();
    let buf_reader = BufReader::new(file);
    println!("{}", count_increments_with_sliding_window(buf_reader).unwrap());
}
