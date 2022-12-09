use std::cmp;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    //#[structopt(parse(from_os_str))]
    //path: std::path::PathBuf,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
struct State {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
struct Bounds {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum BoundsCheck {
    Inbounds,
    Possible,
    Impossible,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum SimulationOutcome {
    HighestPosition(i32),
    NoLanding,
}

///// Solved analytically
//fn solve_analytic(start: State, t: i32) -> State {
//    State {
//        vy: start.vy -t,
//        y:  start.y - (t * (t - 1) /2 ),
//        vx: if start.vx > 0 || t < start.vx {start.vx + t}
//        else if start.vx < 0 || t > start.vx {start.vx - t}
//        else {0}
//        x: start.x + if start.vx > 0 {}
//    }
//}

fn simulate_next(state: State) -> State {
    State {
        x: state.x + state.vx,
        y: state.y + state.vy,
        vy: state.vy - 1,
        vx: if state.vx > 0 {
            state.vx - 1
        } else if state.vx < 0 {
            state.vx + 1
        } else {
            0
        },
    }
}

fn bounds_check(state: &State, bounds: &Bounds) -> BoundsCheck {
    if state.x >= bounds.min_x
        && state.x <= bounds.max_x
        && state.y >= bounds.min_y
        && state.y <= bounds.max_y
    {
        BoundsCheck::Inbounds
    } else if (state.vx >= 0 && bounds.max_x < state.x)
        || (state.vx <= 0 && bounds.min_x > state.x)
        || (state.vy < 0 && bounds.min_y > state.y)
    {
        BoundsCheck::Impossible
    } else {
        BoundsCheck::Possible
    }
}

fn simulate_full_cycle(mut state: State, bounds: Bounds) -> SimulationOutcome {
    let mut highest = state.y;
    loop {
        match bounds_check(&state, &bounds) {
            BoundsCheck::Inbounds => {
                state = simulate_next(state);
                highest = cmp::max(highest, state.y);
                break;
            }
            BoundsCheck::Possible => {
                state = simulate_next(state);
                highest = cmp::max(highest, state.y);
            }
            BoundsCheck::Impossible => {
                return SimulationOutcome::NoLanding;
            }
        }
    }
    SimulationOutcome::HighestPosition(highest)
}

/// The trick with this is that we can ignore x position
/// in the first pass to find candidate y values.
fn sim_until_highest(bounds: Bounds) -> i32 {
    let mut nolanding_count = 0;
    let mut highest = 0;
    let mut vy = 0;
    let mut candidate_vy = Vec::new();
    while nolanding_count < 1000 {
        let state = State {
            x: bounds.min_x,
            y: 0,
            vx: 0,
            vy: vy,
        };
        match simulate_full_cycle(state, bounds) {
            SimulationOutcome::NoLanding => {
                nolanding_count = nolanding_count + 1;
            }
            SimulationOutcome::HighestPosition(_) => {
                nolanding_count = 0;
                candidate_vy.push(vy);
            }
        }
        vy = vy + 1;
    }
    for vx in 0..1000 {
        for vy in &candidate_vy {
            let state = State {
                x: 0,
                y: 0,
                vx: vx,
                vy: *vy,
            };
            if let SimulationOutcome::HighestPosition(i) = simulate_full_cycle(state, bounds) {
                highest = cmp::max(highest, i)
            }
        }
    }
    highest
}

fn sum_all_possible(bounds: Bounds) -> i32 {
    let mut nolanding_count = 0;
    let mut count = 0;
    let mut vy = -1000;
    let mut candidate_vy = Vec::new();
    while nolanding_count < 1000 {
        let state = State {
            x: bounds.min_x,
            y: 0,
            vx: 0,
            vy: vy,
        };
        match simulate_full_cycle(state, bounds) {
            SimulationOutcome::NoLanding => {
                nolanding_count = nolanding_count + 1;
            }
            SimulationOutcome::HighestPosition(_) => {
                nolanding_count = 0;
                candidate_vy.push(vy);
            }
        }
        vy = vy + 1;
    }
    for vx in 0..1000 {
        for vy in &candidate_vy {
            let state = State {
                x: 0,
                y: 0,
                vx: vx,
                vy: *vy,
            };
            if let SimulationOutcome::HighestPosition(_) = simulate_full_cycle(state, bounds) {
                count = count + 1;
            }
        }
    }
    count
}

fn main() {
    let _ = Cli::from_args();
    let bounds = Bounds {
        min_x: 155,
        max_x: 182,
        min_y: -117,
        max_y: -67,
    };
    let high = sim_until_highest(bounds);
    println!("{}", high);
    let count = sum_all_possible(bounds);
    println!("{}", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step() {
        let start = State {
            x: 0,
            y: 0,
            vx: 1,
            vy: 2,
        };
        let next = simulate_next(start);
        assert_eq!(
            next,
            State {
                x: 1,
                y: 2,
                vx: 0,
                vy: 1
            }
        );
    }

    #[test]
    fn test_inbounds_simulation() {
        let start = State {
            x: 0,
            y: 0,
            vx: 7,
            vy: 2,
        };
        let bounds = Bounds {
            min_x: 20,
            max_x: 30,
            min_y: -10,
            max_y: -5,
        };
        let outcome = simulate_full_cycle(start, bounds);
        assert_eq!(outcome, SimulationOutcome::HighestPosition(3));
    }

    #[test]
    fn test_sim_until_highest() {
        let bounds = Bounds {
            min_x: 20,
            max_x: 30,
            min_y: -10,
            max_y: -5,
        };
        let high = sim_until_highest(bounds);
        assert_eq!(high, 45);
    }

    #[test]
    fn test_impossible() {
        let start = State {
            x: 0,
            y: 0,
            vx: 17,
            vy: -4,
        };
        let bounds = Bounds {
            min_x: 20,
            max_x: 30,
            min_y: -10,
            max_y: -5,
        };
        let outcome = simulate_full_cycle(start, bounds);
        assert_eq!(outcome, SimulationOutcome::NoLanding);
    }
}
