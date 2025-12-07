use clap::Parser;
use std::collections::HashSet;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn parse(input: &str) -> (Vec<HashSet<usize>>, HashSet<usize>) {
    (
        input
            .lines()
            .map(|line| {
                let len = line.len();
                line.chars()
                    .enumerate()
                    .filter_map(|(i, x)| match x {
                        '^' => Some(i),
                        _ => None,
                    })
                    .collect()
            })
            .collect(),
        input
            .lines()
            .next()
            .unwrap()
            .chars()
            .enumerate()
            .filter_map(|(i, x)| match x {
                'S' => Some(i),
                _ => None,
            })
            .collect(),
    )
}

fn part1(splitters: &[HashSet<usize>], mut beam: HashSet<usize>) -> (HashSet<usize>, usize) {
    let mut counter = 0;
    for splitter in splitters.iter() {
        counter += beam.intersection(splitter).count();

        let passthrough: HashSet<usize> = beam.difference(splitter).cloned().collect();
        let splits: HashSet<usize> = beam
            .intersection(splitter)
            .flat_map(|x| [x - 1, x + 1].into_iter())
            .collect();
        beam = passthrough.union(&splits).cloned().collect();
    }
    (beam, counter)
}

fn main() {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    let (splitters, beam) = parse(&input);
    println!("Part 1: {:?}", part1(&splitters, beam));
}
