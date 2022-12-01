use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use structopt::StructOpt;
use std::fs;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step() {
    }
}