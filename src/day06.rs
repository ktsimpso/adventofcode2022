use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    chain::Chain,
    prelude::Simple,
    primitive::todo,
    primitive::{end, take_until},
    text, Parser,
};
use clap::ArgMatches;
use std::{cell::LazyCell, collections::BTreeSet};

type ParseOutput = Vec<char>;

pub const DAY_06: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day06",
        "day6 finds unique sets of strings in a message packet.",
        "Path to the input file. Input should be one line for the message",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments {},
        "finds the first set of 4 unique characters in the input string.",
    );
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
    take_until(text::newline())
        .map(|(r, _)| r)
        .then_ignore(end())
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    input
        .windows(4)
        .into_iter()
        .enumerate()
        .find(|(_, chars)| chars.into_iter().cloned().collect::<BTreeSet<char>>().len() == 4)
        .map(|(position, _)| position)
        .expect("valid message")
        + 4
}
