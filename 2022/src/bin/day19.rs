use nom::{
    bytes::complete::tag,
    character::complete::{self, multispace0},
    combinator::map,
    error::ParseError,
    multi::fold_many0,
    sequence::{delimited, separated_pair, tuple},
    IResult,
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use std::{
    cmp::{max, Reverse},
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, VecDeque},
    hash::Hash,
    time::Instant,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    bots: Bots,
    resources: Resources,
    minutes: u32,
    could_move: [bool; 4],
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Resources {
    geode: u32,
    obsidian: u32,
    clay: u32,
    ore: u32,
}

impl Resources {
    fn produce(&self, bots: &Bots) -> Self {
        Self {
            ore: self.ore + bots.ore,
            clay: self.clay + bots.clay,
            obsidian: self.obsidian + bots.obsidian,
            geode: self.geode + bots.geode,
        }
    }
    fn contains(&self, n: u32) -> bool {
        self.geode == n || self.obsidian == n || self.clay == n || self.ore == n
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Bots {
    geode: u32,
    obsidian: u32,
    clay: u32,
    ore: u32,
}

#[derive(Debug, Clone, Hash)]
struct BotCost {
    geode: u32,
    obsidian: u32,
    clay: u32,
    ore: u32,
}

#[derive(Debug, Clone)]
struct Blueprint {
    geode: BotCost,
    obsidian: BotCost,
    clay: BotCost,
    ore: BotCost,
}

impl BotCost {
    fn add(&self, other: &Self) -> Self {
        BotCost {
            ore: self.ore + other.ore,
            clay: self.clay + other.clay,
            obsidian: self.obsidian + other.obsidian,
            geode: self.geode + other.geode,
        }
    }
}

type BlueprintID = u32;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn blueprint(input: &str) -> IResult<&str, (BlueprintID, Blueprint)> {
    let t = |x| ws(tag(x));
    map(
        tuple((
            delimited(t("Blueprint"), complete::u32, t(":")),
            delimited(t("Each ore robot costs"), complete::u32, t("ore.")),
            delimited(t("Each clay robot costs"), complete::u32, t("ore.")),
            delimited(
                t("Each obsidian robot costs"),
                separated_pair(complete::u32, t("ore and"), complete::u32),
                t("clay."),
            ),
            delimited(
                t("Each geode robot costs"),
                separated_pair(complete::u32, t("ore and"), complete::u32),
                t("obsidian."),
            ),
        )),
        |(
            blueprint_id,
            ore_robot_ore_cost,
            clay_robot_ore_cost,
            (obsidian_rebot_ore_cost, obsidian_rebot_clay_cost),
            (geode_robot_ore_cost, geode_rebot_obsidian_cost),
        )| {
            (
                blueprint_id,
                Blueprint {
                    ore: BotCost {
                        ore: ore_robot_ore_cost,
                        clay: 0,
                        obsidian: 0,
                        geode: 0,
                    },
                    clay: BotCost {
                        ore: clay_robot_ore_cost,
                        clay: 0,
                        obsidian: 0,
                        geode: 0,
                    },
                    obsidian: BotCost {
                        ore: obsidian_rebot_ore_cost,
                        clay: obsidian_rebot_clay_cost,
                        obsidian: 0,
                        geode: 0,
                    },
                    geode: BotCost {
                        ore: geode_robot_ore_cost,
                        clay: 0,
                        obsidian: geode_rebot_obsidian_cost,
                        geode: 0,
                    },
                },
            )
        },
    )(input)
}

fn consume(resources: &Resources, cost: &BotCost) -> Option<Resources> {
    match (
        resources.ore.checked_sub(cost.ore),
        resources.clay.checked_sub(cost.clay),
        resources.obsidian.checked_sub(cost.obsidian),
        resources.geode.checked_sub(cost.geode),
    ) {
        (Some(ore), Some(clay), Some(obsidian), Some(geode)) => Some(Resources {
            ore,
            clay,
            obsidian,
            geode,
        }),
        _ => None,
    }
}

fn decisions(state: State, blueprint: &Blueprint, buffer: &mut Vec<State>) {
    let try_ore = consume(&state.resources, &blueprint.ore)
        .filter(|_| !state.could_move[0])
        .map(|resources| State {
            minutes: state.minutes + 1,
            resources: resources.produce(&state.bots),
            could_move: [false; 4],
            bots: Bots {
                ore: state.bots.ore + 1,
                clay: state.bots.clay,
                obsidian: state.bots.obsidian,
                geode: state.bots.geode,
            },
        });
    let try_clay = consume(&state.resources, &blueprint.clay)
        .filter(|_| !state.could_move[1])
        .map(|resources| State {
            minutes: state.minutes + 1,
            resources: resources.produce(&state.bots),
            could_move: [false; 4],
            bots: Bots {
                ore: state.bots.ore,
                clay: state.bots.clay + 1,
                obsidian: state.bots.obsidian,
                geode: state.bots.geode,
            },
        });
    let try_obsidian = consume(&state.resources, &blueprint.obsidian)
        .filter(|_| !state.could_move[2])
        .map(|resources| State {
            minutes: state.minutes + 1,
            resources: resources.produce(&state.bots),
            could_move: [false; 4],
            bots: Bots {
                ore: state.bots.ore,
                clay: state.bots.clay,
                obsidian: state.bots.obsidian + 1,
                geode: state.bots.geode,
            },
        });
    let try_geode = consume(&state.resources, &blueprint.geode)
        .filter(|_| !state.could_move[3])
        .map(|resources| State {
            minutes: state.minutes + 1,
            resources: resources.produce(&state.bots),
            could_move: [false; 4],
            bots: Bots {
                ore: state.bots.ore,
                clay: state.bots.clay,
                obsidian: state.bots.obsidian,
                geode: state.bots.geode + 1,
            },
        });
    let no_bot = Some(State {
        minutes: state.minutes + 1,
        resources: state.resources.produce(&state.bots),
        could_move: [
            state.could_move[0] || try_ore.is_some(),
            state.could_move[1] || try_clay.is_some(),
            state.could_move[2] || try_obsidian.is_some(),
            state.could_move[3] || try_geode.is_some(),
        ],
        bots: state.bots,
    });
    buffer.extend(
        [try_ore, try_clay, try_obsidian, try_geode, no_bot]
            .iter()
            .filter_map(|e| e.clone()),
    )
}

fn search(resources: Resources, blueprint: Blueprint, minutes: u32) -> u32 {
    let mut queue = BinaryHeap::new();
    let init_state = State {
        minutes: 0,
        resources,
        could_move: [false; 4],
        bots: Bots {
            ore: 1,
            clay: 0,
            obsidian: 0,
            geode: 0,
        },
    };
    let mut decision_buffer = Vec::new();
    let mut best_geode = None;
    let mut count: u64 = 0;
    queue.push(init_state);
    while let Some(state) = queue.pop() {
        count += 1;
        count += 1;
        if state.minutes == minutes {
            let previous = best_geode.clone();
            best_geode = best_geode
                .or(Some(state.resources.geode))
                .map(|g| max(state.resources.geode, g));
            continue;
        }

        //println!("state {:?}:, best: {:?}", state, best_geode);
        if let Some(best) = best_geode {
            let remaining_mins = minutes - state.minutes;
            // over-estimate best outcome using arithmetric growth of geode
            // i.e. new bot every minute all the way to the end
            let ideal_geode = state.resources.geode * (remaining_mins + 1)
                + remaining_mins * (remaining_mins + 1) / 2;
            if ideal_geode <= best {
                continue;
            }
        }

        decisions(state, &blueprint, &mut decision_buffer);
        for decision in decision_buffer.drain(..) {
            queue.push(decision);
        }
    }
    best_geode.unwrap()
}

fn score_blueprint(input: &str) -> u32 {
    let start_resource = Resources {
        ore: 0,
        clay: 0,
        obsidian: 0,
        geode: 0,
    };
    let lines: Vec<&str> = input.lines().collect();
    lines
        .par_iter()
        .map(|line| blueprint(line).unwrap())
        .map(|(_, (id, blueprint))| id * search(start_resource.clone(), blueprint, 24))
        .sum()
}

fn score_stolen(input: &str) -> u32 {
    let start_resource = Resources {
        ore: 0,
        clay: 0,
        obsidian: 0,
        geode: 0,
    };
    let lines: Vec<&str> = input.lines().take(3).collect();
    lines
        .par_iter()
        .map(|line| blueprint(line).unwrap())
        .map(|(_, (id, blueprint))| search(start_resource.clone(), blueprint, 32))
        .reduce(|| 1, |a, x| a * x)
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    let start_time = Instant::now();

    println!("solution 1: {}", score_blueprint(&input));
    println!("solution 2: {}", score_stolen(&input));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blueprint_one() {
        let input = "Blueprint 1:
        Each ore robot costs 4 ore.
        Each clay robot costs 2 ore.
        Each obsidian robot costs 3 ore and 14 clay.
        Each geode robot costs 2 ore and 7 obsidian.";
        let (_, (_id, blueprint)) = blueprint(input).unwrap();
        assert_eq!(
            search(
                Resources {
                    ore: 0,
                    clay: 0,
                    obsidian: 0,
                    geode: 0,
                },
                blueprint,
                24,
            ),
            9
        );
    }

    #[test]
    fn test_score() {
        let input = include_str!("../../input/day19-test");
        assert_eq!(score_blueprint(input), 33);
    }
}
