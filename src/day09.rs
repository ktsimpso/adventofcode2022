use adventofcode2022::{
    absolute_difference, parse_between_blank_lines, parse_isize, parse_lines, parse_usize,
    single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just, one_of},
    Parser,
};
use clap::ArgMatches;
use std::{cell::LazyCell, collections::BTreeSet};

type ParseOutput = Vec<Direction>;

pub const DAY_09: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day09",
        "Moves a rope along a path then outputs the number of unqiue positions for the rope's tail.",
        "Path to the input file. Each line should contain a direction for the rope to travel followed by a distance for the rope to travel.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "Uses a rope of length 2.");
    //.with_part2(CommandLineArguments {}, "part 2 help");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    //n: usize,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Up(isize),
    Right(isize),
    Down(isize),
    Left(isize),
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        //n: *args.get_one::<usize>("number").expect("Valid arguments"),
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_direction()).then_ignore(end())
}

fn parse_direction() -> impl Parser<char, Direction, Error = Simple<char>> {
    one_of("URDL")
        .then_ignore(just(" "))
        .then(parse_isize())
        .map(|(direction, length)| match direction {
            'U' => Direction::Up(length),
            'R' => Direction::Right(length),
            'D' => Direction::Down(length),
            'L' => Direction::Left(length),
            _ => unreachable!(),
        })
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let mut movements = BTreeSet::new();
    let mut current_h = (0, 0);
    let mut current_t = (0, 0);

    movements.insert(current_t.clone());

    for step in input.into_iter() {
        match step {
            Direction::Up(up) => current_h.1 += up,
            Direction::Right(right) => current_h.0 += right,
            Direction::Down(down) => current_h.1 -= down,
            Direction::Left(left) => current_h.0 -= left,
        }
        while absolute_difference(current_t.0, current_h.0) > 1
            || absolute_difference(current_t.1, current_h.1) > 1
        {
            if current_t.0 < current_h.0 {
                current_t = (current_t.0 + 1, current_t.1);
            } else if current_t.0 > current_h.0 {
                current_t = (current_t.0 - 1, current_t.1);
            }

            if current_t.1 < current_h.1 {
                current_t = (current_t.0, current_t.1 + 1);
            } else if current_t.1 > current_h.1 {
                current_t = (current_t.0, current_t.1 - 1);
            };
            movements.insert(current_t.clone());
        }
    }

    movements.len()
}
