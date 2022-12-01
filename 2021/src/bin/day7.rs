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

fn fuel_cost(distance: i64) -> i64 {
    assert!(distance >= 0);
    return distance * (distance + 1) / 2;
}

fn crab_game(inputs: Vec<i64>) -> i64 {
    let min_crab = inputs.iter().fold(0, |a, x| cmp::min(a, *x));
    let max_crab = inputs.iter().fold(0, |a, x| cmp::max(a, *x));
    (min_crab..max_crab)
        .map(|midpoint| {
            inputs
                .iter()
                .fold(0, |a, x| a + fuel_cost((x - midpoint).abs()))
        })
        .fold(i64::MAX, |b, attempts| cmp::min(b, attempts))
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let inputs = parse_input(&source)
        .unwrap()
        .iter()
        .map(|x| x.clone() as i64)
        .collect::<Vec<i64>>();
    let minimised_total_distance = crab_game(inputs);
    println!("total distance: {}", minimised_total_distance);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuel_cost() {
        assert_eq!(fuel_cost(1), 1);
        assert_eq!(fuel_cost(2), 1 + 2);
        assert_eq!(fuel_cost(3), 3 + 2 + 1);
        assert_eq!(fuel_cost(4), 4 + 3 + 2 + 1);
        assert_eq!(fuel_cost(5), 5 + 4 + 3 + 2 + 1);
    }
}
