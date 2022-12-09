use adventofcode2022::{
    absolute_difference, parse_isize, parse_lines, single_arg, Command, ParseError, Problem,
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
    let length = single_arg("length", 'l', "The length of the rope")
        .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day09",
        "Moves a rope along a path then outputs the number of unqiue positions for the rope's tail.",
        "Path to the input file. Each line should contain a direction for the rope to travel followed by a distance for the rope to travel.",
        vec![length],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { length: 2 }, "Uses a rope of length 2.")
    .with_part2(CommandLineArguments { length: 10 }, "Uses a rope of lenght 10.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    length: usize,
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
        length: *args.get_one::<usize>("length").expect("Valid arguments"),
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
    let mut chain = vec![(0isize, 0isize); arguments.length];

    chain.last().map(|last| movements.insert(last.clone()));
    let len = chain.len();

    for step in input.into_iter() {
        let (count, mut movement): (isize, Box<dyn FnMut(&mut (isize, isize))>) = match step {
            Direction::Up(up) => (up, Box::new(|h: &mut (isize, isize)| h.1 += 1)),
            Direction::Right(right) => (right, Box::new(|h: &mut (isize, isize)| h.0 += 1)),
            Direction::Down(down) => (down, Box::new(|h: &mut (isize, isize)| h.1 -= 1)),
            Direction::Left(left) => (left, Box::new(|h: &mut (isize, isize)| h.0 -= 1)),
        };

        for _ in 0..count {
            let head = chain.get_mut(0).expect("first item exists");
            movement(head);

            for current_tail_index in 1..len {
                let leader = chain
                    .get(current_tail_index - 1)
                    .expect("valid index")
                    .clone();
                let mut follower = chain.get_mut(current_tail_index).expect("valid index");

                if absolute_difference(follower.0, leader.0) > 1
                    || absolute_difference(follower.1, leader.1) > 1
                {
                    follower.0 += (leader.0 - follower.0).signum();
                    follower.1 += (leader.1 - follower.1).signum();
                }
            }

            chain.last().map(|last| movements.insert(last.clone()));
        }
    }

    movements.len()
}
