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

// struct Directory<'a> {
//     dentries: BTreeMap<&'a str, Dentry>,
// }
//

#[derive(Debug, Clone)]
enum Dentry {
    File(usize),
    Directory,
}

// enum Cd<'a> {
//     Down(&'a str),
//     Up,
// }
//
// enum Command<'a> {
//     Cd(Cd<'a>),
//     Ls,
// }
//
// enum Line {
//     Command(Command),
//     File(File),
//     Directory(Directory),
// }
//
//
// fn parse_line(line: &str) -> Line {
//     let first_word = line.split(" ").next().unwrap();
//     match first_word {
//         "$" => Line::Command(),
//         "dir" => Line::Directory(),
//         _ => Line::File(),
//     }
// }
//
// fn parse_command(command: &'a str) -> Command  {
//     let mut parts = line.split(" ");
//     let first_word = parts.next().unwrap();
//     match first_word {
//         "ls" => Line::Command(Command::Ls),
//         "cd" => Line::Directory(Command::Cd({
//             let arg = parts.next().unwrap();
//             match arg{
//             }
//         }}
//     )),
//         _ => panic!("Unsupported command {}", first_word),
//     }
// }

// fn get_cwd(root: Directory, path: &mut Vec<&str>) -> &mut Directory {
//     let mut dir = root;
//     for name in path.iter() {
//         dir = match dir.dentry.get(name).unwrap(){
//             Dentry::Directory(d) => d,
//             Dentry::File => panic!("unexpected file in path"),
//         }
//     }
// }

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
            match tree.insert(fullpath, Dentry::Directory){
                None => (),
                Some(_) => {panic!("not expecting object")}
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
            match tree.insert(fullpath, Dentry::File(size))  {
                None => (),
                Some(_) => {panic!("not expecting object")}
            };
        }
    }
    println!("{}", line);
    Some(remaining)
}

fn find_dir_size(tree: &BTreeMap<String, Dentry>, dir: &str) -> usize {
    tree.range(std::ops::RangeFrom {
        start: dir.to_owned(),
    })
    .take_while(|(k, _)| k.starts_with(dir))
    .fold(0, |a, (k, v)| {
        
        a + match v {
            Dentry::File(s) => {if (s == &65147){println!("test: {} {:?}", k, v)}; s},
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
    let mut directories = tree
        .iter()
        .filter_map(|(k, v)| match v {
            Dentry::Directory => { Some(find_dir_size(&tree, k))},
            Dentry::File(u) => {if (u == &65147){println!("test dir: {}", k)}; None},
        })
        .filter(|v| v <= &100000)
        .collect::<Vec<usize>>();
    println!("{:?}", directories.iter().sum::<usize>());
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
