use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}
#[derive(PartialEq, Clone, Debug)]
enum Binary {
    One,
    Zero,
}
#[derive(Debug, Clone)]
enum Error {
    ParsedNoneBinaryCharacter,
    EmptyLine,
    IOError,
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::IOError
    }
}

fn parse_line(line: &str) -> Result<Vec<Binary>, Error> {
    Ok(line
        .chars()
        .map(|c| match c {
            '0' => Ok(Binary::Zero),
            '1' => Ok(Binary::One),
            _ => Err(Error::ParsedNoneBinaryCharacter),
        })
        .collect::<Result<Vec<Binary>, Error>>()?)
}

fn get_values(stream: impl BufRead) -> Result<(usize, usize, usize, usize), Error> {
    let mut sanitised_stream: Vec<Vec<Binary>> = stream
        .lines()
        .collect::<Result<Vec<String>, std::io::Error>>()?
        .iter()
        .map(|x| parse_line(x.as_str()))
        .collect::<Result<Vec<Vec<Binary>>, Error>>()?;
    sanitised_stream.retain(|x| x.len() > 0);
    let mut counters = BTreeMap::new();
    for row in sanitised_stream.iter() {
        for (i, c) in row.iter().enumerate() {
            counters.insert(
                i,
                counters.get(&i).unwrap_or(&0)
                    + match c {
                        Binary::One => 1,
                        Binary::Zero => -1,
                    },
            );
        }
    }
    let gamma = counters
        .values()
        .map(|x| if x > &0 { 2 } else { 0 })
        .rev()
        .enumerate()
        .map(|(i, x)| usize::pow(x, i as u32 + 1) / 2)
        .fold(0, |a, x| a + x);
    let epsilon = counters
        .values()
        .map(|x| if x < &0 { 2 } else { 0 })
        .rev()
        .enumerate()
        .map(|(i, x)| usize::pow(x, i as u32 + 1) / 2)
        .fold(0, |a, x| a + x);

    let oxygen = bit_criteria(sanitised_stream.clone(), 0, |x| {
        if x >= 0 {
            Binary::One
        } else {
            Binary::Zero
        }
    })
    .iter()
    .map(|x| match x {
        Binary::One => 2,
        Binary::Zero => 0,
    })
    .rev()
    .enumerate()
    .map(|(i, x)| usize::pow(x, i as u32 + 1) / 2)
    .fold(0, |a, x| a + x);

    let co2 = bit_criteria(sanitised_stream, 0, |x| {
        if x < 0 {
            Binary::One
        } else {
            Binary::Zero
        }
    })
    .iter()
    .map(|x| match x {
        Binary::One => 2,
        Binary::Zero => 0,
    })
    .rev()
    .enumerate()
    .map(|(i, x)| usize::pow(x, i as u32 + 1) / 2)
    .fold(0, |a, x| a + x);

    return Ok((gamma, epsilon, oxygen, co2));
}

fn bit_criteria(
    bits: Vec<Vec<Binary>>,
    level: usize,
    search: fn(i32) -> Binary,
) -> Vec<Binary> {
    let summation: i32 = bits
        .iter()
        .map(|x| match x.get(level).unwrap() {
            Binary::One => 1,
            Binary::Zero => -1,
        })
        .fold(0, |a, x| a + x);
    println!("{} === {:?}", level, bits);
    println!("sum:  {}", summation);
    let firstpass = bits
        .iter()
        .filter(|x| x.get(level).unwrap() == &search(summation))
        .map(|x| x.clone())
        .collect::<Vec<Vec<Binary>>>();
    if firstpass.len() == 1 {
        println!("{:?}", firstpass);
        return firstpass.get(0).unwrap().to_vec();
    }
    bit_criteria(firstpass, level + 1, search)
}

//fn get_gas(stream: impl BufRead) -> Result<(usize, usize), Error> {
//    let sanitised_stream: Vec<Vec<Binary>> = stream
//        .lines()
//        .collect::<Result<Vec<String>, std::io::Error>>()?
//        .iter()
//        .map(|x| parse_line(x.as_str()))
//        .collect::<Result<Vec<Vec<Binary>>, Error>>()?;
//}

fn main() {
    let args = Cli::from_args();
    let mut file = File::open(args.path.as_path()).unwrap();
    let mut buf_reader = BufReader::new(file);
    let (gamma, epsilon, oxygen, co2) = get_values(buf_reader).unwrap();

    println!("gamma={} epsilon={} oxygen={} co2={}", gamma, epsilon, oxygen, co2);
    println!("multiplied = {}", gamma * epsilon);
}
