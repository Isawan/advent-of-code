use std::collections::BTreeMap;
use std::fs;
use structopt::StructOpt;
#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
    steps: u32,
}
type Grid = BTreeMap<(i32, i32), u32>;

fn parse_input(source: &str) -> BTreeMap<(i32, i32), u32> {
    let mut result = BTreeMap::new();
    for (j, line) in source.split('\n').enumerate() {
        for (i, c) in line.chars().enumerate() {
            result.insert((i as i32, j as i32), c.to_digit(10).unwrap());
        }
    }
    result
}

fn step(grid: &mut BTreeMap<(i32, i32), u32>) -> u32 {
    let mut total_flashes = 0;
    // handle increment all octopus
    for v in grid.values_mut() {
        *v = *v + 1
    }
    loop {
        let mut bump = Vec::new();
        let mut step_flash = 0;
        // iterate through grid for flashable octopus
        for (p, v) in grid.iter_mut() {
            if *v <= 9 {
                continue;
            }
            *v = 0;
            step_flash = step_flash + 1;
            for i in -1..=1 {
                for j in -1..=1 {
                    if i == 0 && j == 0 {
                        continue;
                    }
                    bump.push((p.0 + i, p.1 + j));
                }
            }
        }
        // update adjacent flashable octopus;
        for p in bump {
            if let Some(v) = grid.get_mut(&p) {
                if *v != 0 {
                    *v = *v + 1;
                }
            }
        }

        total_flashes = total_flashes + step_flash;
        if step_flash == 0 {
            break;
        }
    }
    total_flashes
}

fn step_rounds(mut grid: Grid, rounds: u32) -> (Grid, u32) {
    let mut count = 0;
    for _ in 0..rounds {
        count = count + step(&mut grid);
    }
    (grid, count)
}

fn search_light_up(mut grid: Grid) -> u32 {
    let mut count = 0;
    loop {
        if grid.values().fold(0, |a, x| a + x) == 0 {
            return count;
        }
        step(&mut grid);
        count = count + 1;
    }
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let grid = parse_input(&source);
    let (_, flashes) = step_rounds(grid, args.steps);
    println!("flashes: {}", flashes);

    let grid = parse_input(&source);
    let steps = search_light_up(grid);
    println!("steps taken: {}", steps);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_function() {
        let mut grid = parse_input(
            "11111\n\
                                    19991\n\
                                    19191\n\
                                    19991\n\
                                    11111\n",
        );
        let expected_grid = parse_input(
            "34543\n\
                                         40004\n\
                                         50005\n\
                                         40004\n\
                                         34543\n",
        );
        let flashes = step(&mut grid);
        assert_eq!(flashes, 9);
        assert_eq!(grid, expected_grid)
    }

    #[test]
    fn test_multiple_rounds() {
        let grid = parse_input(
            "5483143223\n\
                                    2745854711\n\
                                    5264556173\n\
                                    6141336146\n\
                                    6357385478\n\
                                    4167524645\n\
                                    2176841721\n\
                                    6882881134\n\
                                    4846848554\n\
                                    5283751526\n",
        );
        let expected = parse_input(
            "0397666866\n\
                                    0749766918\n\
                                    0053976933\n\
                                    0004297822\n\
                                    0004229892\n\
                                    0053222877\n\
                                    0532222966\n\
                                    9322228966\n\
                                    7922286866\n\
                                    6789998766\n",
        );
        let (grid, flashes) = step_rounds(grid, 100);
        assert_eq!(flashes, 1656);
        assert_eq!(grid, expected);
    }

    #[test]
    fn test_search_light_up() {
        let grid = parse_input(
            "5483143223\n\
                                    2745854711\n\
                                    5264556173\n\
                                    6141336146\n\
                                    6357385478\n\
                                    4167524645\n\
                                    2176841721\n\
                                    6882881134\n\
                                    4846848554\n\
                                    5283751526\n",
        );
        let steps = search_light_up(grid);
        assert_eq!(steps, 195);
    }
}
