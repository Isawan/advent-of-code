use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    path: std::path::PathBuf,
}

fn parser(input: &str) -> (Vec<(i64, i64)>, Vec<i64>) {
    let (fresh_ingredient_ranges, available_ingredient) = input.split_once("\n\n").unwrap();
    let fresh_ingredients_ranges = fresh_ingredient_ranges
        .lines()
        .map(|line| line.split_once('-').unwrap())
        .map(|(a, b)| (a.parse().unwrap(), b.parse().unwrap()))
        .collect();
    let available_ingredient = available_ingredient
        .lines()
        .map(|x| x.parse().unwrap())
        .collect();
    (fresh_ingredients_ranges, available_ingredient)
}

fn part1((ranges, ingredients): &(Vec<(i64, i64)>, Vec<i64>)) -> usize {
    ingredients
        .iter()
        .filter(|&ingredient| {
            ranges
                .iter()
                .any(|(lower, higher)| ingredient >= lower && ingredient <= higher)
        })
        .count()
}

fn gobble((low, high): (i64, i64), max: Option<i64>) -> (i64, Option<i64>) {
    match max {
        None => (high - low + 1, Some(high)),
        Some(m) if high <= m => (0, Some(m)),
        Some(m) if high > m && low <= m => (high - m, Some(high)),
        Some(m) if high > m && low > m => (high - low + 1, Some(high)),
        _ => panic!("impossible"),
    }
}

fn part2(ingredients: &Vec<(i64, i64)>) -> i64 {
    let mut ingredients = ingredients.clone();
    ingredients.sort();
    ingredients
        .iter()
        .fold((0, None), |(counter, max), &x| {
            println!("{:?} {:?} {:?}", counter, max, x);
            let (c, n) = gobble(x, max);
            (counter + c, n)
        })
        .0
}

fn main() -> () {
    let cli = Cli::parse();
    let input = std::fs::read_to_string(cli.path).expect("Failed to read input file");
    let input = parser(&input);
    println!("Part 1: {}", part1(&input));
    println!("Part 2: {}", part2(&input.0));
}
