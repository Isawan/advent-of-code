use std::iter::zip;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct Monkey {
    items: Vec<i64>,
    operation: Box<dyn Fn(i64) -> i64>,
    division_number: i64,
    true_monkey_index: usize,
    false_monkey_index: usize,
}
struct Item(usize);

impl Monkey {
    fn modulo(&mut self, modulo: i64) {
        self.items = self.items.iter().map(|x| x % modulo).collect();
    }
    fn inspect(&mut self) {
        self.items = self
            .items
            .iter()
            .map(|item| (self.operation)(*item))
            .collect();
    }
    fn calm(&mut self) {
        self.items = self.items.iter().map(|item| item / 3).collect();
    }
    fn throw_items(&mut self) -> Vec<(usize, i64)> {
        let items_to_throw = self.items.drain(..).collect::<Vec<i64>>();
        items_to_throw
            .iter()
            .map(|x| -> (usize, i64) {
                (
                    if x % self.division_number == 0 {
                        self.true_monkey_index
                    } else {
                        self.false_monkey_index
                    },
                    *x,
                )
            })
            .collect()
    }
}

fn monkey_round(mut monkeys: Vec<Monkey>) -> (Vec<Monkey>, Vec<u64>) {
    let mut inspection_count = vec![0; monkeys.len()];
    for i in 0..monkeys.len() {
        let monkey = &mut monkeys[i];
        monkey.inspect();
        monkey.calm();
        for (index, item) in monkey.throw_items() {
            inspection_count[i] = inspection_count[i] + 1;
            monkeys[index].items.push(item)
        }
    }
    (monkeys, inspection_count)
}
fn worried_monkey_round(mut monkeys: Vec<Monkey>) -> (Vec<Monkey>, Vec<u64>) {
    let mut inspection_count = vec![0; monkeys.len()];
    let modulo = monkeys.iter().fold(1, |a,x| a * x.division_number);
    for i in 0..monkeys.len() {
        let monkey = &mut monkeys[i];
        monkey.inspect();
        monkey.modulo(modulo);
        for (index, item) in monkey.throw_items() {
            inspection_count[i] = inspection_count[i] + 1;
            monkeys[index].items.push(item)
        }
    }
    (monkeys, inspection_count)
}


fn monkey_business(mut monkeys: Vec<Monkey>, round: impl Fn(Vec<Monkey>) -> (Vec<Monkey>, Vec<u64>), rounds: u32) -> u64 {
    let mut total_inspections = vec![0; monkeys.len()];
    for i in 0..rounds {
        let round_inspections;
        (monkeys, round_inspections) = round(monkeys);
        total_inspections = zip(total_inspections, round_inspections)
            .map(|(a, b)| a + b)
            .collect();
    }
    total_inspections.sort_by(|a, b| b.cmp(a));
    println!("{:?}", total_inspections);
    total_inspections.iter().take(2).fold(1, |a, x| a * x)
}


fn main() {
    let start = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();

    println!("monkey business: {}", monkey_business(input_monkeys(),monkey_round, 20));
    println!("monkey business: {}", monkey_business(input_monkeys(),worried_monkey_round, 10000));
    println!("time: {}", start.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use std::iter::Inspect;

    use super::*;

    #[test]
    fn test_monkey_round() {
        let monkeys = init_monkeys();
        let (monkeys, _) = monkey_round(monkeys);
        assert_eq!(monkeys[0].items, vec![20, 23, 27, 26]);
        assert_eq!(monkeys[1].items, vec![2080, 25, 167, 207, 401, 1046]);
        assert_eq!(monkeys[2].items, vec![]);
        assert_eq!(monkeys[3].items, vec![]);
    }

    #[test]
    fn multiple_monkey_rounds() {
        let mut monkeys = init_monkeys();
        let mut total_inspections = vec![0; monkeys.len()];
        for i in 0..20 {
            let round_inspections;
            (monkeys, round_inspections) = monkey_round(monkeys);
            total_inspections = zip(total_inspections, round_inspections)
                .map(|(a, b)| a + b)
                .collect();
        }
        assert_eq!(total_inspections[0], 101);
        assert_eq!(total_inspections[1], 95);
        assert_eq!(total_inspections[2], 7);
        assert_eq!(total_inspections[3], 105);
    }

    #[test]
    fn test_monkey_business() {
        let monkey_business = monkey_business(init_monkeys(), monkey_round, 20);
        assert_eq!(monkey_business, 10605);
    }

    #[test]
    fn test_worried_monkey_business() {
        let monkeys = init_monkeys();
        let (_, round_inspection) = worried_monkey_round(monkeys);
        assert_eq!(round_inspection[0], 2);
        assert_eq!(round_inspection[1], 4);
        assert_eq!(round_inspection[2], 3);
        assert_eq!(round_inspection[3], 6);

        let mut monkeys = init_monkeys();
        let mut total_inspections = vec![0; monkeys.len()];
        for i in 0..20 {
            let round_inspections;
            (monkeys, round_inspections) = worried_monkey_round(monkeys);
            total_inspections = zip(total_inspections, round_inspections)
                .map(|(a, b)| a + b)
                .collect();
        }
        assert_eq!(total_inspections[0], 99);
        assert_eq!(total_inspections[1], 97);
        assert_eq!(total_inspections[2], 8);
        assert_eq!(total_inspections[3], 103);
    }

    fn init_monkeys() -> Vec<Monkey> {
        vec![
            Monkey {
                items: vec![79, 98],
                operation: Box::new(|old| old * 19),
                division_number: 23,
                true_monkey_index: 2,
                false_monkey_index: 3,
            },
            Monkey {
                items: vec![54, 65, 75, 74],
                operation: Box::new(|old| old + 6),
                division_number: 19,
                true_monkey_index: 2,
                false_monkey_index: 0,
            },
            Monkey {
                items: vec![79, 60, 97],
                operation: Box::new(|old| old * old),
                division_number: 13,
                true_monkey_index: 1,
                false_monkey_index: 3,
            },
            Monkey {
                items: vec![74],
                operation: Box::new(|old| old + 3),
                division_number: 17,
                true_monkey_index: 0,
                false_monkey_index: 1,
            },
        ]
    }

}

fn input_monkeys() -> Vec<Monkey> {
    vec![
        Monkey {
            items: vec![54, 82, 90, 88, 86, 54],
            operation: Box::new(|old| old * 7),
            division_number: 11,
            true_monkey_index: 2,
            false_monkey_index: 6,
        },
        Monkey {
            items: vec![91, 65],
            operation: Box::new(|old| old * 13),
            division_number: 5,
            true_monkey_index: 7,
            false_monkey_index: 4,
        },
        Monkey {
            items: vec![62, 54, 57, 92, 83, 63, 63],
            operation: Box::new(|old| old + 1),
            division_number: 7,
            true_monkey_index: 1,
            false_monkey_index: 7,
        },
        Monkey {
            items: vec![67, 72, 68],
            operation: Box::new(|old| old * old),
            division_number: 2,
            true_monkey_index: 0,
            false_monkey_index: 6,
        },
        Monkey {
            items: vec![68, 89, 90, 86, 84, 57, 72, 84],
            operation: Box::new(|old| old + 7),
            division_number: 17,
            true_monkey_index: 3,
            false_monkey_index: 5,
        },
        Monkey {
            items: vec![79, 83, 64, 58],
            operation: Box::new(|old| old + 6),
            division_number: 13,
            true_monkey_index: 3,
            false_monkey_index: 0,
        },
        Monkey {
            items: vec![96, 72, 89, 70, 88],
            operation: Box::new(|old| old + 4),
            division_number: 3,
            true_monkey_index: 1,
            false_monkey_index: 2,
        },
        Monkey {
            items: vec![79],
            operation: Box::new(|old| old + 8),
            division_number: 19,
            true_monkey_index: 4,
            false_monkey_index: 5,
        },
    ]
}
