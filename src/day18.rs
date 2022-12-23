use adventofcode2022::{flag_arg, parse_isize, parse_lines, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::ArgMatches;
use itertools::Itertools;
use std::{cell::LazyCell, collections::HashSet};

type ParseOutput = Vec<Point3d>;

pub const DAY_18: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let air = flag_arg(
        "air",
        'a',
        "Account for air bubble when finding the exposed surface area.",
    );
    let problem = Problem::new(
        "day18",
        "Finds the total surface area of lava drops.",
        "Path to the input file. File should consist of one 3d coordinate of lava per line.",
        vec![air],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments { air: false },
        "Finds the total exposed surface area of the lava drops.",
    )
    .with_part2(
        CommandLineArguments { air: true },
        "Finds the total exposed surface area but accounts for air bubbles.",
    );
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    air: bool,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        air: *args.get_one::<bool>("air").unwrap_or(&false),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Point3d {
    x: isize,
    y: isize,
    z: isize,
}

impl Point3d {
    fn get_adjacent_points(&self) -> Vec<Point3d> {
        vec![
            Point3d {
                x: self.x + 1,
                y: self.y,
                z: self.z,
            },
            Point3d {
                x: self.x - 1,
                y: self.y,
                z: self.z,
            },
            Point3d {
                x: self.x,
                y: self.y + 1,
                z: self.z,
            },
            Point3d {
                x: self.x,
                y: self.y - 1,
                z: self.z,
            },
            Point3d {
                x: self.x,
                y: self.y,
                z: self.z + 1,
            },
            Point3d {
                x: self.x,
                y: self.y,
                z: self.z - 1,
            },
        ]
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_point()).then_ignore(end())
}

fn parse_point() -> impl Parser<char, Point3d, Error = Simple<char>> {
    parse_isize()
        .then_ignore(just(","))
        .then(parse_isize())
        .then_ignore(just(","))
        .then(parse_isize())
        .map(|((x, y), z)| Point3d { x, y, z })
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let points = input.iter().cloned().collect::<HashSet<_>>();
    let (min_x, max_x) = match input.iter().map(|point| point.x).minmax() {
        itertools::MinMaxResult::NoElements => (0, 0),
        itertools::MinMaxResult::OneElement(minmax) => (minmax, minmax),
        itertools::MinMaxResult::MinMax(min, max) => (min, max),
    };
    let (min_y, max_y) = match input.iter().map(|point| point.y).minmax() {
        itertools::MinMaxResult::NoElements => (0, 0),
        itertools::MinMaxResult::OneElement(minmax) => (minmax, minmax),
        itertools::MinMaxResult::MinMax(min, max) => (min, max),
    };
    let (min_z, max_z) = match input.iter().map(|point| point.z).minmax() {
        itertools::MinMaxResult::NoElements => (0, 0),
        itertools::MinMaxResult::OneElement(minmax) => (minmax, minmax),
        itertools::MinMaxResult::MinMax(min, max) => (min, max),
    };

    let mut known_escape_points = HashSet::new();
    let mut known_trap_points = HashSet::new();

    points
        .iter()
        .map(|this| {
            let adjacents = this
                .get_adjacent_points()
                .into_iter()
                .filter(|point| points.contains(point))
                .collect::<HashSet<_>>();

            let air_count = if arguments.air {
                this.get_adjacent_points()
                    .into_iter()
                    .filter(|point| !adjacents.contains(&point))
                    .filter(|air| {
                        let mut visited = HashSet::new();
                        let result = is_air_bubble(
                            air,
                            &min_x,
                            &max_x,
                            &min_y,
                            &max_y,
                            &min_z,
                            &max_z,
                            &points,
                            &known_escape_points,
                            &known_trap_points,
                            &mut visited,
                        );

                        if result {
                            known_trap_points.extend(visited.into_iter());
                        } else {
                            known_escape_points.extend(visited.into_iter());
                        }
                        result
                    })
                    .count()
            } else {
                0
            };

            6 - (adjacents.len() + air_count)
        })
        .sum()
}

fn is_air_bubble(
    point: &Point3d,
    min_x: &isize,
    max_x: &isize,
    min_y: &isize,
    max_y: &isize,
    min_z: &isize,
    max_z: &isize,
    lava_points: &HashSet<Point3d>,
    known_escape_points: &HashSet<Point3d>,
    known_trap_points: &HashSet<Point3d>,
    visited: &mut HashSet<Point3d>,
) -> bool {
    visited.insert(point.clone());
    if known_escape_points.contains(&point) {
        return false;
    } else if known_trap_points.contains(&point) {
        return true;
    }

    if point.x >= *max_x
        || point.y >= *max_y
        || point.z >= *max_z
        || point.x <= *min_x
        || point.y <= *min_y
        || point.z <= *min_z
    {
        return false;
    }

    let new_points = point
        .get_adjacent_points()
        .into_iter()
        .filter(|point| !(visited.contains(point) || lava_points.contains(&point)))
        .collect::<Vec<_>>();

    new_points.into_iter().all(|point| {
        is_air_bubble(
            &point,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
            lava_points,
            known_escape_points,
            known_trap_points,
            visited,
        )
    })
}
