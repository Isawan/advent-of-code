use std::fs;
use std::num::ParseIntError;
use structopt::StructOpt;
use std::collections::BTreeMap;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,

    #[structopt(short="d", default_value="80")]
    days: u32,
}

fn parse_input(input: &str) -> Result<Vec<u64>,ParseIntError> {
    input.trim().split(',').map(|n| n.parse::<u64>()).collect()
}
fn group_by_timer(timers: Vec<u64>) -> BTreeMap<u64,u64>{
    let mut counter = BTreeMap::new();
    for timer in timers {
        *counter.entry(timer).or_insert(0) += 1;
    }
    counter
}

fn step_timer(old_counter: BTreeMap<u64,u64>) -> BTreeMap<u64,u64> {
    let mut new_counter = BTreeMap::new();
    for (&timers, &count) in old_counter.range(1..){
        new_counter.insert(timers-1, count);
    }
    // handle breeding cycle reset
    if let Some(count) = old_counter.get(&0) {
        *new_counter.entry(6).or_insert(0) += count
    }
    // handle breeding
    if let Some(count) = old_counter.get(&0) {
        *new_counter.entry(8).or_insert(0) += count;
    }
    new_counter
}

fn main() {
    let args = Cli::from_args();
    let days = args.days;
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let inputs = parse_input(&source).unwrap();
    let mut counter = group_by_timer(inputs);
    for _ in 0..days {
        counter = step_timer(counter);
    }
    let number_of_fish = counter.values().fold(0, |a,x| a + x);
    println!("Fishes: {}", number_of_fish);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by() {
        let v: Vec<u64> = vec![1,2,3,6,4,3,2];
        let result = group_by_timer(v);
        let mut expected_counter = BTreeMap::new();
        expected_counter.insert(1,1);
        expected_counter.insert(2,2);
        expected_counter.insert(3,2);
        expected_counter.insert(4,1);
        expected_counter.insert(6,1);
        assert_eq!(result, expected_counter);
    }

    #[test]
    fn test_counter_decrease_per_step() {
        let mut timers_count = BTreeMap::new();
        timers_count.insert(1,1);
        timers_count.insert(2,1);
        timers_count.insert(3,2);
        timers_count.insert(4,1);
        let mut expected_step_timers_count = BTreeMap::new();
        expected_step_timers_count.insert(0,1);
        expected_step_timers_count.insert(1,1);
        expected_step_timers_count.insert(2,2);
        expected_step_timers_count.insert(3,1);
        let next_step_timers = step_timer(timers_count);
        assert_eq!(next_step_timers, expected_step_timers_count);
    }

    #[test]
    fn test_counter_breeding() {
        let mut timers_count = BTreeMap::new();
        timers_count.insert(0,4);
        let mut expected_step_timers_count = BTreeMap::new();
        expected_step_timers_count.insert(6,4);
        expected_step_timers_count.insert(8,4);
        let next_step_timers = step_timer(timers_count);
        assert_eq!(next_step_timers, expected_step_timers_count);
    }

    #[test]
    fn test_lifecycle() {
        let mut timers_count = BTreeMap::new();
        timers_count.insert(1,1);
        timers_count.insert(2,1);
        timers_count.insert(3,2);
        timers_count.insert(4,1);
        let mut expected_step_timers_count = BTreeMap::new();
        expected_step_timers_count.insert(0,3);
        expected_step_timers_count.insert(1,5);
        expected_step_timers_count.insert(2,3);
        expected_step_timers_count.insert(3,2);
        expected_step_timers_count.insert(4,2);
        expected_step_timers_count.insert(5,1);
        expected_step_timers_count.insert(6,5);
        expected_step_timers_count.insert(7,1);
        expected_step_timers_count.insert(8,4);

        for _ in 0..18 {
            timers_count = step_timer(timers_count)
        }
        assert_eq!(timers_count, expected_step_timers_count);
    }

}
