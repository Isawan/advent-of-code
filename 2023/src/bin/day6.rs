use std::{convert::TryInto, fs::read, time::Instant};

use clap::Parser;

// Solved via maths using quadratic formula

fn bounds(t: f64, d: f64) -> (f64, f64) {
    let q = ((t * t) - (4.0 * d)).sqrt();
    ((t - q) / 2.0, (t + q) / 2.0)
}

fn integer_between_bounds(t: f64, d: f64) -> i64 {
    let (n, m) = bounds(t, d);
    m.floor() as i64 - n.ceil() as i64 + 1
}

fn main() {
    let start = Instant::now();
    let g = integer_between_bounds;
    println!(
        "Part 1: {}",
        g(41.0, 214.0) * g(96.0, 1789.0) * g(88.0, 1127.0) * g(94.0, 1055.0)
    );
    println!("Part 2: {}", g(41968894.0, 214178911271055.0));
    println!("Time elapsed: {:?}", start.elapsed());
}
