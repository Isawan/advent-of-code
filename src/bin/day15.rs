use std::array::IntoIter;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::BinaryHeap;
use std::fs;
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

#[allow(dead_code)]
fn visualize(grid: &Grid) -> String {
    let (max_x, max_y) = corner(grid);
    let mut s = String::new();
    for y in 0..max_y + 1 {
        for x in 0..max_x + 1 {
            let v = grid.get(&(x, y)).unwrap();
            s.push(char::from_digit(*v, 10).unwrap());
        }
        s.push('\n');
    }
    s
}

fn expand_map(grid: &Grid) -> Grid {
    let mut result = BTreeMap::new();
    let corner = corner(&grid);
    for ((i, j), v) in grid {
        for k in 0..5 {
            for l in 0..5 {
                let v = v + l as u32 + k as u32 - 1;
                result.insert(
                    (k * (corner.0 + 1) + i, l * (corner.1 + 1) + j),
                    (v % 9) + 1,
                );
            }
        }
    }

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
    heap.push(Reverse(start));
    'outer: while let Some(Reverse(state)) = heap.pop() {
        if visited.contains(&state.pos) {
            continue;
        }
        for (i, j) in IntoIter::new([(0, -1), (0, 1), (-1, 0), (1, 0)]) {
            let x = &state.pos.0 + &i;
            let y = &state.pos.1 + &j;
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
                visited.insert(state.pos);
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
    println!("minimum risk: {}", path[path.len() - 1].1);
    let expanded_grid = expand_map(&grid);
    let path = risk_search(&expanded_grid);
    println!("minimum risk on expanded: {}", path[path.len() - 1].1);
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
    fn test_small_expand_map() {
        let input = parse_input("8\n");
        let exp = parse_input(
            "89123\n\
             91234\n\
             12345\n\
             23456\n\
             34567\n",
        );
        let output = expand_map(&input);
        assert_eq!(output, exp);
    }

    #[test]
    fn test_expand_map() {
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
        let exp = parse_input(
            "11637517422274862853338597396444961841755517295286\n\
             13813736722492484783351359589446246169155735727126\n\
             21365113283247622439435873354154698446526571955763\n\
             36949315694715142671582625378269373648937148475914\n\
             74634171118574528222968563933317967414442817852555\n\
             13191281372421239248353234135946434524615754563572\n\
             13599124212461123532357223464346833457545794456865\n\
             31254216394236532741534764385264587549637569865174\n\
             12931385212314249632342535174345364628545647573965\n\
             23119445813422155692453326671356443778246755488935\n\
             22748628533385973964449618417555172952866628316397\n\
             24924847833513595894462461691557357271266846838237\n\
             32476224394358733541546984465265719557637682166874\n\
             47151426715826253782693736489371484759148259586125\n\
             85745282229685639333179674144428178525553928963666\n\
             24212392483532341359464345246157545635726865674683\n\
             24611235323572234643468334575457944568656815567976\n\
             42365327415347643852645875496375698651748671976285\n\
             23142496323425351743453646285456475739656758684176\n\
             34221556924533266713564437782467554889357866599146\n\
             33859739644496184175551729528666283163977739427418\n\
             35135958944624616915573572712668468382377957949348\n\
             43587335415469844652657195576376821668748793277985\n\
             58262537826937364893714847591482595861259361697236\n\
             96856393331796741444281785255539289636664139174777\n\
             35323413594643452461575456357268656746837976785794\n\
             35722346434683345754579445686568155679767926678187\n\
             53476438526458754963756986517486719762859782187396\n\
             34253517434536462854564757396567586841767869795287\n\
             45332667135644377824675548893578665991468977611257\n\
             44961841755517295286662831639777394274188841538529\n\
             46246169155735727126684683823779579493488168151459\n\
             54698446526571955763768216687487932779859814388196\n\
             69373648937148475914825958612593616972361472718347\n\
             17967414442817852555392896366641391747775241285888\n\
             46434524615754563572686567468379767857948187896815\n\
             46833457545794456865681556797679266781878137789298\n\
             64587549637569865174867197628597821873961893298417\n\
             45364628545647573965675868417678697952878971816398\n\
             56443778246755488935786659914689776112579188722368\n\
             55172952866628316397773942741888415385299952649631\n\
             57357271266846838237795794934881681514599279262561\n\
             65719557637682166874879327798598143881961925499217\n\
             71484759148259586125936169723614727183472583829458\n\
             28178525553928963666413917477752412858886352396999\n\
             57545635726865674683797678579481878968159298917926\n\
             57944568656815567976792667818781377892989248891319\n\
             75698651748671976285978218739618932984172914319528\n\
             56475739656758684176786979528789718163989182927419\n\
             67554889357866599146897761125791887223681299833479\n",
        );
        let output = expand_map(&input);
        assert_eq!(output, exp);
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
        assert_eq!(output[0], ((0, 0), 10));
        assert_eq!(output[1], ((0, 1), 20));
    }
}
