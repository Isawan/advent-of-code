use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{max, min};
use std::collections::BTreeSet;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
struct SensorInfo {
    sensor: (i32, i32),
    beacon: (i32, i32),
}

impl From<((i32, i32), (i32, i32))> for SensorInfo {
    fn from(tuple: ((i32, i32), (i32, i32))) -> Self {
        SensorInfo {
            sensor: tuple.0,
            beacon: tuple.1,
        }
    }
}

impl SensorInfo {
    fn cover_distance(&self) -> i32 {
        (self.sensor.0 - self.beacon.0).abs() + (self.sensor.1 - self.beacon.1).abs()
    }
    fn intersect_y(&self, y: i32) -> Option<(i32, i32)> {
        // handle not intersecting
        let min_distance = (self.sensor.1 - y).abs();
        if min_distance > self.cover_distance() {
            return None;
        }

        let spread = self.cover_distance() - min_distance;
        Some((self.sensor.0 - spread, self.sensor.0 + spread))
    }
}

fn beacon_covered_by_span(beacon: (i32, i32), y: i32, span: (i32, i32)) -> bool {
    // does not lay on intersection line
    if beacon.1 != y {
        return false;
    }
    span.0 <= beacon.0 && span.1 >= beacon.0
}

fn calc_cover(input: &str, intersect_y: i32) -> i32 {
    let infos = parse(input);
    let beacons = infos
        .iter()
        .map(|i| i.beacon)
        .collect::<BTreeSet<(i32, i32)>>();
    let spans = infos.iter().filter_map(|i| i.intersect_y(intersect_y));
    let grouped_spans = group_spans(spans);
    let area_covered_by_span: i32 = grouped_spans
        .iter()
        .map(|(start, end)| end - start + 1)
        .sum();
    let mut beacons_in_span = 0;
    for beacon in beacons.iter() {
        for span in grouped_spans.iter() {
            if beacon_covered_by_span(*beacon, intersect_y, *span) {
                beacons_in_span = beacons_in_span + 1;
                continue;
            }
        }
    }
    area_covered_by_span - beacons_in_span
}

fn calc_spot(input: &str, most: i32) -> i64 {
    let infos = parse(input);
    for y in 0..=most {
        let spans = infos.iter().filter_map(|i| i.intersect_y(y));
        let restricted_spans = restrict_spans(spans, most);
        let grouped_spans = group_spans(restricted_spans);
        if grouped_spans.len() != 1 {
            return (y as i64) + ((grouped_spans[0].1 + 1) as i64) * 4_000_000;
        }
    }
    unreachable!();
}

fn restrict_spans(
    spans: impl Iterator<Item = (i32, i32)>,
    most: i32,
) -> impl Iterator<Item = (i32, i32)> {
    spans.filter_map(move |(x0, x1)| {
        if x1 < 0 {
            return None;
        }
        if x0 > most {
            return None;
        }
        Some((max(x0, 0), min(x1, most)))
    })
}

fn group_spans(spans: impl Iterator<Item = (i32, i32)>) -> Vec<(i32, i32)> {
    let mut sorted_spans = Vec::with_capacity(128);
    sorted_spans.extend(spans);
    sorted_spans.sort();
    let mut end_spans = Vec::with_capacity(128);
    for span in sorted_spans.iter() {
        let (start, end) = *span;
        if let Some(last_span @ (last_start, last_end)) = end_spans.pop() {
            if start <= last_end {
                // overlap. merge them
                end_spans.push((last_start, max(end, last_end)));
            } else if last_end + 1 == start {
                // ends touching. merge them
                end_spans.push((last_start, end));
            } else {
                // keep seperate
                end_spans.push(last_span);
                end_spans.push(span.clone());
            }
        } else {
            end_spans.push(span.clone());
        }
    }
    end_spans
}

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"Sensor at x=(-?\d+), y=(-?\d+): closest beacon is at x=(-?\d+), y=(-?\d+)")
            .unwrap();
}

fn parse(input: &str) -> Vec<SensorInfo> {
    RE.captures_iter(input)
        .map(|m| {
            (
                (
                    m.get(1).unwrap().as_str().parse::<i32>().unwrap(),
                    m.get(2).unwrap().as_str().parse::<i32>().unwrap(),
                ),
                (
                    m.get(3).unwrap().as_str().parse::<i32>().unwrap(),
                    m.get(4).unwrap().as_str().parse::<i32>().unwrap(),
                ),
            )
        })
        .map(|x| x.into())
        .collect()
}

fn main() {
    let start_time = Instant::now();
    let args = Cli::from_args();
    let input = std::fs::read_to_string(args.path.as_path()).unwrap();
    println!("solution 1: {}", calc_cover(&input, 2_000_000));
    println!("solution 2: {}", calc_spot(&input, 4_000_000));
    println!("time: {}", start_time.elapsed().as_micros());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = include_str!("../../input/day15-test");
        let infos = parse(input);
        assert_eq!(infos.len(), 14);
    }

    #[test]
    fn test_intersect() {
        let info = SensorInfo {
            sensor: (8, 7),
            beacon: (2, 10),
        };
        let (x0, x1) = info.intersect_y(2).unwrap();
        assert_eq!(x0, 4);
        assert_eq!(x1, 12);
    }

    #[test]
    fn test_examples() {
        let input = include_str!("../../input/day15-test");
        let infos = parse(input);
        let spans = infos.iter().filter_map(|i| i.intersect_y(10));
        let grouped_spans = group_spans(spans);
        assert_eq!(grouped_spans, vec![(-2, 24)]);
    }

    #[test]
    fn test_cover() {
        let input = include_str!("../../input/day15-test");
        assert_eq!(calc_cover(input, 9), 25);
        assert_eq!(calc_cover(input, 10), 26);
        assert_eq!(calc_cover(input, 11), 28);
    }

    #[test]
    fn test_spot_search() {
        let input = include_str!("../../input/day15-test");
        assert_eq!(calc_spot(input, 20), 56000011);
    }
}
