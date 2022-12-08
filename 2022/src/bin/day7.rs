use itertools::Itertools;
use std::collections::BTreeMap;
use std::ops::Bound::Excluded;
use std::ops::Bound::Included;
use std::ops::RangeBounds;
use std::ops::RangeFrom;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone)]
enum Dentry {
    File(usize),
    Directory,
}

fn perform<'a>(
    tree: &mut BTreeMap<String, Dentry>,
    stack: &mut Vec<&'a str>,
    remaining: &'a str,
) -> Option<&'a str> {
    let dir_size = 0;
    let pos = match remaining.find("\n") {
        Some(t) => t,
        None => return None,
    };
    let (line, remaining) = remaining.split_at(pos);
    let (_, remaining) = remaining.split_at(1); // ignore leading newline
    let mut parts = line.split(' ');
    let first_word = parts.next().unwrap();
    match first_word {
        "$" => {
            let command = parts.next().unwrap();
            match command {
                "cd" => {
                    let arg = parts.next().unwrap();
                    match arg {
                        "/" => {
                            stack.clear();
                        }
                        ".." => {
                            stack.pop().unwrap();
                        }
                        name => {
                            stack.push(name);
                        }
                    }
                }
                // handle in the next run, we will rely on the hack that
                // only command that produces an output is ls so we don't need
                // track state
                "ls" => (),
                _ => {
                    panic!("unexpected command: {}", command)
                }
            }
        }
        "dir" => {
            let filename = parts.next().unwrap();
            let mut fullpath = String::new();
            if stack.len() != 0 {
                fullpath.push('/');
            }
            fullpath.push_str(&stack.join("/"));
            fullpath.push('/');
            fullpath.push_str(filename);
            match tree.insert(fullpath, Dentry::Directory) {
                None => (),
                Some(_) => {
                    panic!("not expecting object")
                }
            };
        }
        _ => {
            let size = first_word.parse::<usize>().expect("Expected int");
            let filename = parts.next().unwrap();
            let mut fullpath = String::new();
            if stack.len() != 0 {
                fullpath.push('/');
            }
            fullpath.push_str(&stack.join("/"));
            fullpath.push('/');
            fullpath.push_str(filename);
            match tree.insert(fullpath, Dentry::File(size)) {
                None => (),
                Some(_) => {
                    panic!("not expecting object")
                }
            };
        }
    }
    Some(remaining)
}

fn find_dir_size(tree: &BTreeMap<String, Dentry>, dir: &str) -> usize {
    let mut leading_dir = dir.to_owned();
    leading_dir.push('/');
    tree.range(std::ops::RangeFrom {
        start: leading_dir.clone(),
    })
    .take_while(|(k, _)| k.starts_with(&leading_dir))
    .fold(0, |a, (k, v)| {
        a + match v {
            Dentry::File(s) => s,
            Dentry::Directory => &0,
        }
    })
}

fn main() {
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let mut tree = BTreeMap::new();
    let mut stack = Vec::new();
    let mut remain = input.as_str();
    loop {
        remain = match perform(&mut tree, &mut stack, remain) {
            Some(t) => t,
            None => break,
        };
    }
    let sum_dir = tree
        .iter()
        .filter_map(|(k, v)| match v {
            Dentry::Directory => Some(find_dir_size(&tree, k)),
            Dentry::File(u) => None,
        })
        .filter(|v| v <= &100000)
        .sum::<usize>();

    println!("sum: {:?}", sum_dir);

    let used = find_dir_size(&tree, "");
    let max = 70_000_000;
    let free = max - used;
    println!("free: {}", free);
    let required_to_free = 30_000_000 - free;
    println!("required: {}", required_to_free);
    let best_pick = tree
        .iter()
        .filter_map(|(k, v)| match v {
            Dentry::Directory => Some(find_dir_size(&tree, k)),
            Dentry::File(u) => None,
        })
        .filter(|v| v >= &required_to_free)
        .sorted()
        .next()
        .unwrap();
    println!("best pick: {}", best_pick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_start() {
        let mut tree = BTreeMap::new();
        let mut stack = Vec::new();
        let result = perform(
            &mut tree,
            &mut stack,
            "$ cd /\n\
                 $ ls\n\
                 dir a\n\
                 14848514 b.txt\n\
                 8504156 c.dat\n\
                 dir d\n",
        )
        .unwrap();
        assert_eq!(
            result,
            "$ ls\n\
             dir a\n\
             14848514 b.txt\n\
             8504156 c.dat\n\
             dir d\n"
        );
        assert_eq!(stack.len(), 0);

        let result = perform(&mut tree, &mut stack, result).unwrap();
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_example() {
        let mut tree = BTreeMap::new();
        let mut stack = Vec::new();
        let example = "$ cd /\n\
                       $ ls\n\
                       dir a\n\
                       14848514 b.txt\n\
                       8504156 c.dat\n\
                       dir d\n\
                       $ cd a\n\
                       $ ls\n\
                       dir e\n\
                       29116 f\n\
                       2557 g\n\
                       62596 h.lst\n\
                       $ cd e\n\
                       $ ls\n\
                       584 i\n\
                       $ cd ..\n\
                       $ cd ..\n\
                       $ cd d\n\
                       $ ls\n\
                       4060174 j\n\
                       8033020 d.log\n\
                       5626152 d.ext\n\
                       7214296 k\n";
        let mut remain = example;
        loop {
            remain = match perform(&mut tree, &mut stack, remain) {
                Some(t) => t,
                None => break,
            };
        }
        assert_eq!(find_dir_size(&tree, "/a/e"), 584);
        assert_eq!(find_dir_size(&tree, "/a"), 94853);
        assert_eq!(find_dir_size(&tree, "/d"), 24933642);
    }
}
