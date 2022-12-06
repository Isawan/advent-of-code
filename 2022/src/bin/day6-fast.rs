/// Experimental test to make it go faster using a bit mask
use itertools::Itertools;
use structopt::StructOpt;
use std::time::Instant;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn ascii_letter_to_bitfield(c: u8) -> u32 {
    return 1 << (c & 0b00011111)
}

fn unique_count(buffer: &[u8] ) -> u32 {
    let mut result: u32 =  0;
    for byte in buffer.iter() {
        result = result | ascii_letter_to_bitfield(*byte);
    }
    result.count_ones()
}

fn get_start(buffer: &str, distinct: usize) -> Option<usize> {
    buffer
        .as_bytes()
        .windows(distinct)
        .position(|window| unique_count(window) as usize == distinct)
        .map(|x| x + distinct)
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();
    for _ in (0..10000) {

    let start_of_packet = get_start(&input, 4).unwrap();
    // println!("Start of packet: {}", start_of_packet);

    let start_of_message = get_start(&input, 14).unwrap();
    // println!("Start of message: {}", start_of_message);

    }
    let elapse = start_time.elapsed();
    println!("start of message: {}s {}ms", elapse.as_secs(), elapse.subsec_millis());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_start() {
        assert_eq!(get_start("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 4), Some(7));
    }
}
