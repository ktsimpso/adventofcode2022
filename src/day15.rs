use adventofcode2022::{
    absolute_difference, parse_isize, parse_lines, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::ArgMatches;
use itertools::Itertools;
use std::{
    cell::LazyCell,
    cmp::{max, min},
    collections::VecDeque,
};

type ParseOutput = Vec<Sensor>;

pub const DAY_15: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let line = single_arg(
        "line",
        'l',
        "The line to scan and get the number of impossible spaces",
    )
    .value_parser(clap::value_parser!(isize));
    let area = single_arg(
        "area",
        'a',
        "The max area to look for an open space. Returns the tuning frequency of the space",
    )
    .value_parser(clap::value_parser!(isize))
    .conflicts_with("line");
    let problem = Problem::new(
        "day15",
        "Finds statistics about the results of our sensor's beacon targets.",
        "Path to the input file. Each line should have a sensor, it's postition, and the position of the beacon that it's closest too.",
        vec![line, area],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { scanner_mode: ScannerMode::ScanLine(2_000_000) }, "Finds the number of positions where the signal can not exist for y = 2_000_000")
    .with_part2(CommandLineArguments { scanner_mode: ScannerMode::ScanArea(4_000_000) }, "Finds the tuning frequency for the area of 4_000_000");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub enum ScannerMode {
    ScanLine(isize),
    ScanArea(isize),
}

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    scanner_mode: ScannerMode,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    let line = args
        .get_one::<isize>("line")
        .map(|constant| ScannerMode::ScanLine(*constant));
    let area = args
        .get_one::<isize>("area")
        .map(|constant| ScannerMode::ScanArea(*constant));
    let scanner_mode = match (line, area) {
        (Some(y), None) => y,
        (None, Some(y)) => y,
        _ => unreachable!(),
    };
    CommandLineArguments { scanner_mode }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Point {
    x: isize,
    y: isize,
}

#[derive(Debug, Clone)]
pub struct Sensor {
    location: Point,
    beacon: Point,
}

#[derive(Debug, Clone)]
struct Range {
    start: isize,
    end: isize,
}

impl Range {
    fn overlap_with_other(&self, other: &Range) -> isize {
        min(self.end, other.end) - max(self.start, other.start) + 1
    }
}

impl Sensor {
    fn get_beacon_distance(&self) -> isize {
        self.get_distance(&self.beacon)
    }

    fn get_distance(&self, point: &Point) -> isize {
        absolute_difference(self.location.x, point.x)
            + absolute_difference(self.location.y, point.y)
    }

    fn get_impossible_points_for_y(&self, target_y: isize) -> Option<Range> {
        let max_distance = self.get_beacon_distance();
        let target_distance = self.get_distance(&Point {
            x: self.location.x,
            y: target_y,
        });

        if target_distance <= max_distance {
            Some(Range {
                start: self.location.x - (max_distance - target_distance),
                end: self.location.x + (max_distance - target_distance),
            })
        } else {
            None
        }
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_sensor()).then_ignore(end())
}

fn parse_sensor() -> impl Parser<char, Sensor, Error = Simple<char>> {
    just("Sensor at ")
        .ignore_then(parse_point())
        .then_ignore(just(": closest beacon is at "))
        .then(parse_point())
        .map(|(location, beacon)| Sensor { location, beacon })
}

fn parse_point() -> impl Parser<char, Point, Error = Simple<char>> {
    just("x=")
        .ignore_then(parse_isize())
        .then_ignore(just(", y="))
        .then(parse_isize())
        .map(|(x, y)| Point { x, y })
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> isize {
    match arguments.scanner_mode {
        ScannerMode::ScanLine(y) => {
            let (_, ranges) = find_ranges_for_y(&input, y);
            ranges
                .into_iter()
                .map(|range| range.end - range.start)
                .sum()
        }
        ScannerMode::ScanArea(search) => {
            let mut y = 0;
            while y <= search {
                let (min_overlap, ranges) = find_ranges_for_y(&input, y);
                if ranges.len() == 2 {
                    return (ranges.get(0).expect("At least 1").end + 1) * 4_000_000 + y;
                }

                y += match min_overlap {
                    Some(overlap) => max(1, overlap / 2),
                    None => 1,
                }
            }
            unreachable!()
        }
    }
}

fn find_ranges_for_y(sensors: &Vec<Sensor>, y: isize) -> (Option<isize>, VecDeque<Range>) {
    sensors
        .iter()
        .filter_map(|sensor| sensor.get_impossible_points_for_y(y))
        .sorted_by(|a, b| a.start.cmp(&b.start))
        .fold(
            (None, VecDeque::<Range>::new()),
            |(min_overlap, mut acc), range| match acc.back_mut() {
                Some(last_range) => {
                    let overlap = last_range.overlap_with_other(&range);
                    if overlap >= 0 {
                        last_range.end = max(last_range.end, range.end);
                    } else {
                        acc.push_back(range);
                    };
                    let new_overlap = match min_overlap {
                        Some(old_overlap) => Some(min(old_overlap, overlap)),
                        None => Some(overlap),
                    };
                    (new_overlap, acc)
                }
                None => {
                    acc.push_back(range);
                    (None, acc)
                }
            },
        )
}
