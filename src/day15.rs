use adventofcode2022::{
    absolute_difference, parse_between_blank_lines, parse_isize, parse_lines, parse_usize,
    single_arg, Command, ParseError, Problem,
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
    collections::{HashSet, VecDeque},
};

type ParseOutput = Vec<Sensor>;

pub const DAY_15: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day15",
        "Finds statistics about the results of our sensor's beacon targets.",
        "Path to the input file. Each line should have a sensor, it's postition, and the position of the beacon that it's closest too.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "Finds the number of positions where the signal can not exist for y = 2_000_000");
    //.with_part2(CommandLineArguments { }, "part 2 help");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    //n: usize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        //n: *args.get_one::<usize>("number").expect("Valid arguments"),
    }
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

impl Sensor {
    fn get_beacon_distance(&self) -> isize {
        self.get_distance(&self.beacon)
    }

    fn get_distance(&self, point: &Point) -> isize {
        absolute_difference(self.location.x, point.x)
            + absolute_difference(self.location.y, point.y)
    }

    fn get_impossible_points_for_y(&self, target_y: isize) -> HashSet<Point> {
        let max_distance = self.get_beacon_distance();
        let mut queue = VecDeque::from([Point {
            x: self.location.x,
            y: target_y,
        }]);
        let mut results = HashSet::new();
        let mut visited = HashSet::new();

        while let Some(target) = queue.pop_front() {
            if visited.contains(&target) {
                continue;
            }
            visited.insert(target.clone());

            if self.get_distance(&target) <= max_distance && target != self.beacon {
                queue.push_back(Point {
                    x: target.x + 1,
                    y: target.y,
                });
                queue.push_back(Point {
                    x: target.x - 1,
                    y: target.y,
                });
                results.insert(target);
            }
        }
        results
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

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    input
        .into_iter()
        .map(|sensor| sensor.get_impossible_points_for_y(2_000_000))
        .fold(HashSet::new(), |mut acc, next| {
            acc.extend(next.into_iter());
            acc
        })
        .len()
}
