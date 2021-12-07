use std::cmp;
use std::fs;
use std::num::ParseIntError;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn parse_input(input: &str) -> Result<Vec<u64>, ParseIntError> {
    input.trim().split(',').map(|n| n.parse::<u64>()).collect()
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let inputs = parse_input(&source)
        .unwrap()
        .iter()
        .map(|x| x.clone() as i64)
        .collect::<Vec<i64>>();
    let min_crab = inputs.iter().fold(0, |a, x| cmp::min(a, *x));
    let max_crab = inputs.iter().fold(0, |a, x| cmp::max(a, *x));
    //let avg = inputs.iter().fold(0, |a, x| a + x) / (inputs.len() as i64);
    let shortest_total_distance = (min_crab..max_crab)
        .map(|midpoint| inputs.iter().fold(0, |a, x| a + (x - midpoint).abs()))
        .fold(100000000000, |b, attempts| cmp::min(b, attempts));
    println!("total distance: {}", shortest_total_distance);
}

#[cfg(test)]
mod tests {
    use super::*;
}
