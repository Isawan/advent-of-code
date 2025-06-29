use std::env;
use std::fs;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let filename = args
        .get(1)
        .expect("Please provide a filename as an argument");
    let contents = fs::read_to_string(filename).expect("Could not read file");

    let mut v1: Vec<i32> = Vec::new();
    let mut v2: Vec<i32> = Vec::new();
    for line in contents.lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        let p = parts.as_slice();
        match p {
            [p1, p2] => {
                v1.push(p1.parse().expect("Failed"));
                v2.push(p2.parse().expect("Failed"));
            }
            x => println!("err parsing: {x:?}"),
        }
    }

    v1.sort();
    v2.sort();

    let score: u32 = v1.iter().zip(v2.iter()).map(|(i, j)| i.abs_diff(*j)).sum();
    println!("part1: {score}");

    let score: i32 = v1
        .iter()
        .map(|x| x * (v2.iter().filter(|i| *i == x).count() as i32))
        .sum();

    println!("part2: {score}");
}
