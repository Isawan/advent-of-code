use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs;
use structopt::StructOpt;
#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
}

