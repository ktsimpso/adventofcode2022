use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just, one_of, todo},
    text, Parser,
};
use clap::ArgMatches;
use std::cell::LazyCell;

type ParseOutput = (Vec<Vec<Option<char>>>, Vec<(usize, usize, usize)>);

pub const DAY_05: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day05",
        "Takes the current stacks as well as crane instructions for how to move boxes between stacks. Then returns the top of each stack after all moves",
        "Path to the input file. File should consist of the stacks, followed by a blank line, then newline delimited move instructions.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "The crane moves each box one at a time.");
    //.with_part2(CommandLineArguments {}, "part 2 help");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {}

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
    parse_lines(parse_crate_line())
        .then_ignore(parse_crate_line_numbers())
        .then(parse_lines(parse_instruction()))
        .then_ignore(end())
}

fn parse_instruction() -> impl Parser<char, (usize, usize, usize), Error = Simple<char>> {
    just("move ")
        .ignore_then(parse_usize())
        .then_ignore(just(" from "))
        .then(parse_usize())
        .then_ignore(just(" to "))
        .then(parse_usize())
        .map(|((m, f), t)| (m, f, t))
}

fn parse_crate_line_numbers() -> impl Parser<char, Vec<usize>, Error = Simple<char>> {
    just(" ").ignore_then(
        parse_usize()
            .separated_by(just("   "))
            .then_ignore(just(" \n\n")),
    )
}

fn parse_crate_line() -> impl Parser<char, Vec<Option<char>>, Error = Simple<char>> {
    parse_crate()
        .map(|c| Some(c))
        .or(just("   ").map(|_| None))
        .separated_by(just(' '))
}

fn parse_crate() -> impl Parser<char, char, Error = Simple<char>> {
    let alphabet = ('A'..='Z').collect::<String>();
    just('[')
        .ignore_then(one_of(alphabet))
        .then_ignore(just(']'))
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> String {
    let mut stacks = convert_to_stacks(input.0);

    input
        .1
        .into_iter()
        .map(|(m, f, t)| (m, f - 1, t - 1))
        .for_each(|(m, f, t)| {
            let from = stacks.get_mut(f).expect("stack exists");
            let mut popped = (0..m)
                .map(|_| from.pop().expect("Non empty stack"))
                .collect::<Vec<char>>();
            let to = stacks.get_mut(t).expect("stack exists");
            to.append(&mut popped);
        });

    stacks
        .into_iter()
        .filter_map(|stack| stack.last().cloned())
        .collect()
}

fn convert_to_stacks(crates: Vec<Vec<Option<char>>>) -> Vec<Vec<char>> {
    let mut stacks = vec![Vec::new(); crates.len()];

    crates.into_iter().for_each(|row| {
        row.into_iter()
            .enumerate()
            .filter_map(|(pos, crat)| crat.map(|c| (pos, c)))
            .for_each(|(pos, c)| {
                stacks
                    .get_mut(pos)
                    .into_iter()
                    .for_each(|stack| stack.push(c))
            })
    });

    stacks.iter_mut().for_each(|stack| stack.reverse());

    stacks
}
