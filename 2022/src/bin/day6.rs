use itertools::Itertools;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn get_start(buffer: &str, distinct: usize) -> Option<usize> {
    buffer
        .as_bytes()
        .windows(distinct)
        .position(|window| window.iter().unique().count() == distinct)
        .map(|x| x + distinct)
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_of_packet = get_start(&input, 4).unwrap();
    println!("Start of packet: {}", start_of_packet);

    let start_of_message = get_start(&input, 14).unwrap();
    println!("Start of message: {}", start_of_message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_start() {
        assert_eq!(get_start("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 4), Some(7));
    }
}
