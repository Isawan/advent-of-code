use std::array::IntoIter;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::fs;
use std::io::stdout;
use std::io::Write;
use std::rc::Rc;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

type Grid = BTreeMap<(i32, i32), u32>;

fn corner(grid: &Grid) -> (i32, i32) {
    let corner = grid.iter().next_back().unwrap();
    *corner.0
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone)]
struct SearchState {
    total_risk: u32,
    pos: (i32, i32),
    prev: Option<Rc<SearchState>>,
}

fn parse_input(source: &str) -> Grid {
    let mut result = BTreeMap::new();
    for (j, line) in source.split('\n').enumerate() {
        for (i, c) in line.chars().enumerate() {
            result.insert((i as i32, j as i32), c.to_digit(10).unwrap());
        }
    }
    result
}

fn flatten_search_state(state: Rc<SearchState>) -> Vec<((i32, i32), u32)> {
    let mut result = Vec::new();
    let mut current_state = &Some(state);
    loop {
        match current_state {
            Some(s) => {
                result.push((s.pos, s.total_risk));
                current_state = &s.prev;
            }
            None => {
                break;
            }
        }
    }
    result.reverse();
    result
}

fn risk_search(grid: &Grid) -> Vec<((i32, i32), u32)> {
    let mut heap = BinaryHeap::new();
    let mut visited = BTreeSet::new();
    let start = Rc::new(SearchState {
        total_risk: 0,
        pos: (0, 0),
        prev: None,
    });
    let mut path = Vec::new();
    let corner = corner(&grid);
    let mut k = 0;
    visited.insert(start.pos.clone());
    heap.push(Reverse(start));
    'outer: while let Some(Reverse(state)) = heap.pop() {
        for (i, j) in IntoIter::new([(0, -1), (0, 1), (-1, 0), (1, 0)]) {
            let x = &state.pos.0 + &i;
            let y = &state.pos.1 + &j;
            println!("    {:?} {:?}", state.pos, state.total_risk);
            if visited.contains(&(x, y)) {
                continue;
            }
            if let Some(risk) = grid.get(&(x, y)) {
                let probe = Rc::new(SearchState {
                    total_risk: state.total_risk + *risk,
                    pos: (x, y),
                    prev: Some(state.clone()),
                });
                if (x, y) == corner {
                    path = flatten_search_state(probe);
                    break 'outer;
                }
                heap.push(Reverse(probe));
                visited.insert((state.pos.0, state.pos.1));
            }
        }
        k = k + 1;
    }
    path
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let grid = parse_input(&source);
    let path = risk_search(&grid);
    println!("minimum risk: {}", path[path.len()-1].1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        let input = parse_input(
            "1163751742\n\
                                 1381373672\n\
                                 2136511328\n\
                                 3694931569\n\
                                 7463417111\n\
                                 1319128137\n\
                                 1359912421\n\
                                 3125421639\n\
                                 1293138521\n\
                                 2311944581\n",
        );
        let mut path = risk_search(&input);
        assert_eq!(path.pop().unwrap().1, 40);
    }

    #[test]
    fn test_search_state() {
        let input = SearchState {
            total_risk: 20,
            pos: (0, 1),
            prev: Some(Rc::new(SearchState {
                total_risk: 10,
                pos: (0, 0),
                prev: None,
            })),
        };
        let output = flatten_search_state(Rc::new(input));
        assert_eq!(output[0], ((0,0), 10));
        assert_eq!(output[1], ((0,1), 20));
    }
}
