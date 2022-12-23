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
use std::{cell::LazyCell, collections::HashSet};

type ParseOutput = Vec<Point3d>;

pub const DAY_18: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day18",
        "Finds the total surface area of lava drops.",
        "Path to the input file. File should consist of one 3d coordinate of lava per line.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments {},
        "Finds to total exposed surface area of the lava drops.",
    );
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

    points
        .iter()
        .map(|this| {
            let adjacents = this
                .get_adjacent_points()
                .into_iter()
                .filter(|point| points.contains(point))
                .collect::<HashSet<_>>();

            6 - (adjacents.len())
        })
        .sum()
}
