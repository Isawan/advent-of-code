use ndarray::{arr1, arr2, Array1, Array2};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn number_to_segment() -> Array2<usize> {
    arr2(&[
        [1, 1, 1, 0, 1, 1, 1], // 0
        [0, 0, 1, 0, 0, 1, 0], // 1
        [1, 0, 1, 1, 1, 0, 1], // 2
        [1, 0, 1, 1, 0, 1, 1], // 3
        [0, 1, 1, 1, 0, 1, 0], // 4
        [1, 1, 0, 1, 0, 1, 1], // 5
        [1, 1, 0, 1, 1, 1, 1], // 6
        [1, 0, 1, 0, 0, 1, 0], // 7
        [1, 1, 1, 1, 1, 1, 1], // 8
        [1, 1, 1, 1, 0, 1, 1], // 9
    ])
}

fn correct_segments() -> HashMap<usize, BTreeSet<usize>> {
    let mut segment_map = HashMap::new();
    let digits = &[
        [1, 1, 1, 0, 1, 1, 1], // 0
        [0, 0, 1, 0, 0, 1, 0], // 1
        [1, 0, 1, 1, 1, 0, 1], // 2
        [1, 0, 1, 1, 0, 1, 1], // 3
        [0, 1, 1, 1, 0, 1, 0], // 4
        [1, 1, 0, 1, 0, 1, 1], // 5
        [1, 1, 0, 1, 1, 1, 1], // 6
        [1, 0, 1, 0, 0, 1, 0], // 7
        [1, 1, 1, 1, 1, 1, 1], // 8
        [1, 1, 1, 1, 0, 1, 1], // 9
    ];
    for (i, digit_segments) in digits.iter().enumerate() {
        let mut segment_set = BTreeSet::new();
        for (j, segment) in digit_segments.iter().enumerate() {
            if *segment == 1 {
                segment_set.insert(j);
            }
        }
        segment_map.insert(i, segment_set);
    }
    segment_map
}

fn letter_to_index(letter: char) -> usize {
    match letter {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        _ => panic!("Unexpected char {}", letter),
    }
}

fn parse_line(line: &str) -> (Vec<BTreeSet<usize>>, Vec<BTreeSet<usize>>) {
    let index = line.find('|').unwrap();
    let (signal_patterns_text, four_digit_text) = line.split_at(index);
    let four_digit_text = four_digit_text[1..].trim();
    let signal_patterns_text = signal_patterns_text.trim();

    let signal_patterns = signal_patterns_text
        .split(' ')
        .map(|tokens| tokens.chars().map(letter_to_index).collect())
        .collect();
    let four_digit = four_digit_text
        .split(' ')
        .map(|tokens| tokens.chars().map(letter_to_index).collect())
        .collect();

    (signal_patterns, four_digit)
}

fn parse_input(source: &str) -> (Vec<Vec<BTreeSet<usize>>>, Vec<Vec<BTreeSet<usize>>>) {
    let mut signal_patterns = Vec::new();
    let mut four_digits = Vec::new();
    for line in source.trim().split('\n') {
        let (s, f) = parse_line(line);
        signal_patterns.push(s);
        four_digits.push(f);
    }
    (signal_patterns, four_digits)
}

fn count_easy_digits(four_digits_list: Vec<Vec<BTreeSet<usize>>>) -> usize {
    four_digits_list
        .iter()
        .flatten()
        .filter(|digit| match digit.len() {
            2 => true,
            4 => true,
            3 => true,
            7 => true,
            _ => false,
        })
        .count()
}

fn solve_number_to_set(digits: Vec<BTreeSet<usize>>) -> HashMap<usize, BTreeSet<usize>>{
    let one = digits
        .iter()
        .filter(|digit| digit.len() == 2)
        .next()
        .unwrap();
    let four = digits
        .iter()
        .filter(|digit| digit.len() == 4)
        .next()
        .unwrap();
    let seven = digits
        .iter()
        .filter(|digit| digit.len() == 3)
        .next()
        .unwrap();
    let eight = digits
        .iter()
        .filter(|digit| digit.len() == 7)
        .next()
        .unwrap();
    //find two
    let two;
    {
        let two_s1 = (four - one);
        let two_c1 = digits
            .iter()
            .filter(|x| x.is_subset(&two_s1))
            .collect::<Vec<&BTreeSet<usize>>>();
        assert_eq!(two_c1.len(), 3);
        let two_c2 = two_c1
            .iter()
            .filter(|x| !x.is_subset(&one))
            .map(|x| (*x).clone())
            .collect::<Vec<BTreeSet<usize>>>();
        assert_eq!(two_c2.len(), 1);
        two = two_c2.iter().next().unwrap().clone();
    }

    let mut result = HashMap::new();
    result.insert(0,one.clone()   );
    result.insert(3,four.clone()  );
    result.insert(6,seven.clone() );
    result.insert(7,eight.clone() );
    result.insert(1,two.clone()   );
    result
}

fn main() {
    let args = Cli::from_args();
    let source = fs::read_to_string(args.path.as_path()).unwrap();
    let (s, f) = parse_input(&source);
    let count = count_easy_digits(f);
    println!("easy digit count: {}", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let (s, f) = parse_input("dbc gfecab afcdg dfebcag bd dgbe bcaeg dcefab ecgadb agcbd | acdgb gbcda gdecfba bacge\n\
bacdegf aefbdc ebf fdbcag edbfa gdaeb acfdb cdegbf face fe | ebf ecdabf fcbad afcdbg\n\
cabgde gd becgfd dgfe cebgf gfdeacb fdbac bcgaef bgdfc gdc | cbfad dg dgef ecfbdg\n");
        let mut t1 = BTreeSet::new();
        t1.insert(3);
        t1.insert(2);
        t1.insert(1);
        let mut t2 = BTreeSet::new();
        t2.insert(2);
        t2.insert(4);
        t2.insert(1);
        t2.insert(6);
        t2.insert(5);
        let mut t3 = BTreeSet::new();
        t3.insert(3);
        t3.insert(6);
        t3.insert(4);
        t3.insert(5);
        assert_eq!(s[0][0], t1);
        assert_eq!(s[2][4], t2);
        assert_eq!(f[2][2], t3);
    }

    #[test]
    fn test_single_solver() {
        let (s, _) = parse_input(
            "acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab | cdfeb fcadb cdfeb cdbaf\n",
        );
        let r = solve_number_to_set(s[0].clone());
        let mut expected = HashMap::new();
        let f = |x: &str| x.chars().map(letter_to_index).collect::<BTreeSet<usize>>();

        expected.insert(0, f("cagedb" ));
        expected.insert(1, f("ab"     ));
        expected.insert(2, f("gcdfa"  ));
        expected.insert(3, f("fbcad"  ));
        expected.insert(4, f("eafb"   ));
        expected.insert(5, f("cdfbe"  ));
        expected.insert(6, f("cdfgeb" ));
        expected.insert(7, f("dab"    ));
        expected.insert(8, f("acedgfb"));
        expected.insert(9, f("cefabd" ));

        assert_eq!(r,expected);
    }
}
