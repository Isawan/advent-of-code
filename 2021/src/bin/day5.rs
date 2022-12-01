use regex::Regex;
use std::cmp;
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Coord {
    x: usize,
    y: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct Grid {
    entries: Vec<usize>,
    width: usize,
    height: usize,
}

fn parse_line(source: &str) -> (Option<(Coord, Coord)>, &str) {
    let regex = Regex::new(r"^(\d+),(\d+) -> (\d+),(\d+)\n?").unwrap();
    if let Some(captures) = regex.captures(source) {
        let mut matches = captures.iter();

        let whole_match = matches.next().unwrap().unwrap();
        let c0 = matches
            .next()
            .unwrap()
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();
        let c1 = matches
            .next()
            .unwrap()
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();
        let c2 = matches
            .next()
            .unwrap()
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();
        let c3 = matches
            .next()
            .unwrap()
            .unwrap()
            .as_str()
            .parse::<usize>()
            .unwrap();

        (
            Some((Coord { x: c0, y: c1 }, Coord { x: c2, y: c3 })),
            &source[whole_match.end()..],
        )
    } else {
        (None, source)
    }
}

fn parse_coordinates(source: &str) -> (Vec<(Coord, Coord)>, &str) {
    let mut source = source;
    let mut coords_list = Vec::new();
    while source != "" {
        let result = parse_line(source);
        source = result.1;
        coords_list.push(result.0.unwrap());
    }
    (coords_list, "")
}

fn make_big_grid(coords: &Vec<(Coord, Coord)>) -> Grid {
    let max_number = coords
        .iter()
        .map(|cs| cmp::max(cmp::max(cs.0.x, cs.0.y), cmp::max(cs.1.x, cs.1.y)))
        .fold(0, |a, x| cmp::max(a, x));
    let width = max_number + 1;
    let height = max_number + 1;
    Grid {
        entries: vec![0; width * height],
        width: width,
        height: height,
    }
}

fn draw_line(grid: &mut Grid, coords: (Coord, Coord)) {
    if coords.0.x == coords.1.x {
        let x = coords.0.x;
        let miny = cmp::min(coords.0.y, coords.1.y);
        let maxy = cmp::max(coords.0.y, coords.1.y);
        for y in miny..(maxy + 1) {
            grid.increment(x, y);
        }
    } else if coords.0.y == coords.1.y {
        let y = coords.0.y;
        let minx = cmp::min(coords.0.x, coords.1.x);
        let maxx = cmp::max(coords.0.x, coords.1.x);
        for x in minx..(maxx + 1) {
            grid.increment(x, y);
        }
    } else {
        let mut y = coords.0.y;
        let mut x = coords.0.x;
        let max_steps = cmp::max(coords.1.x, coords.0.x) - cmp::min(coords.1.x, coords.0.x);
        let mut steps = 0;
        while steps <= max_steps {
            grid.increment(x, y);
            steps += 1;
            if steps > max_steps {
                break;
            }
            x = if coords.1.x > coords.0.x {
                x + 1
            } else {
                x - 1
            };
            y = if coords.1.y > coords.0.y {
                y + 1
            } else {
                y - 1
            };
        }
    }
}

fn count_overlap(grid: &Grid) -> usize {
    grid.entries.iter().filter(|c| c > &&1).count()
}

impl Grid {
    fn increment(&mut self, x: usize, y: usize) {
        self.entries[y * self.width + x] += 1;
    }
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let (coord_pairs, _) = parse_coordinates(&source);
    let mut grid = make_big_grid(&coord_pairs);
    for c in coord_pairs {
        draw_line(&mut grid, c);
    }
    let count = count_overlap(&grid);
    println!("overlaps: {}", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lines_single_line() {
        let parsed_line = parse_line("111,222 -> 33,4\n");
        assert_eq!((&parsed_line.0).unwrap().0.x, 111usize);
        assert_eq!((&parsed_line.0).unwrap().0.y, 222usize);
        assert_eq!((&parsed_line.0).unwrap().1.x, 33usize);
        assert_eq!((&parsed_line.0).unwrap().1.y, 4usize);
        assert_eq!(&parsed_line.1, &"");
    }
    #[test]
    fn test_parse_lines_multiline() {
        let parsed_line = parse_line("111,222 -> 33,4\n5,6 -> 7,8");
        assert_eq!((&parsed_line.0).unwrap().0.x, 111usize);
        assert_eq!((&parsed_line.0).unwrap().0.y, 222usize);
        assert_eq!((&parsed_line.0).unwrap().1.x, 33usize);
        assert_eq!((&parsed_line.0).unwrap().1.y, 4usize);
        assert_eq!(parsed_line.1, "5,6 -> 7,8");
    }
    #[test]
    fn test_parse_lines_remaining_source() {
        let (all_coords, _) = parse_coordinates("111,222 -> 33,4\n5,6 -> 7,8");
        assert_eq!(
            all_coords,
            vec![
                (Coord { x: 111, y: 222 }, Coord { x: 33, y: 4 }),
                (Coord { x: 5, y: 6 }, Coord { x: 7, y: 8 })
            ]
        );
    }
    #[test]
    fn test_big_grid() {
        let coords = vec![
            (Coord { x: 111, y: 222 }, Coord { x: 33, y: 4 }),
            (Coord { x: 5, y: 6 }, Coord { x: 7, y: 8 }),
        ];
        let grid = make_big_grid(&coords);
        assert_eq!(grid.width, 222 + 1);
        assert_eq!(grid.height, 222 + 1);
        assert_eq!(grid.entries.len(), (222 + 1) * (222 + 1));
    }
    #[test]
    fn test_draw() {
        let coords = vec![
            (Coord { x: 1, y: 0 }, Coord { x: 1, y: 2 }),
            (Coord { x: 0, y: 1 }, Coord { x: 2, y: 1 }),
        ];
        let mut grid = make_big_grid(&coords);
        for c in coords {
            draw_line(&mut grid, c);
        }
        assert_eq!(grid.entries, &[0, 1, 0, 1, 2, 1, 0, 1, 0]);
    }
    #[test]
    fn test_count_overlap() {
        let grid = Grid {
            entries: vec![0, 1, 0, 1, 2, 1, 0, 1, 0],
            width: 3,
            height: 3,
        };
        let count = count_overlap(&grid);
        assert_eq!(count, 1);
    }
    #[test]
    fn test_diagonal_draws() {
        let coords = vec![
            (Coord { x: 0, y: 0 }, Coord { x: 2, y: 2 }),
            (Coord { x: 2, y: 0 }, Coord { x: 0, y: 2 }),
        ];
        let mut grid = make_big_grid(&coords);
        for c in coords {
            draw_line(&mut grid, c);
        }
        assert_eq!(grid.entries, &[1, 0, 1, 0, 2, 0, 1, 0, 1]);
    }
}
