use adventofcode2022::{single_arg, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, take_until},
    text, Parser,
};
use clap::ArgMatches;
use std::{cell::LazyCell, collections::BTreeSet};

type ParseOutput = Vec<char>;

pub const DAY_06: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let number = single_arg("number", 'n', "The number of unique characters to find")
        .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day06",
        "day6 finds unique sets of strings in a message packet.",
        "Path to the input file. Input should be one line for the message",
        vec![number],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments { n: 4 },
        "Finds the first set of 4 unique characters in the input string.",
    )
    .with_part2(
        CommandLineArguments { n: 14 },
        "Finds the first set of 14 unique characters in the input string.",
    );
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    n: usize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        n: *args.get_one::<usize>("number").expect("Valid arguments"),
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
        .windows(arguments.n)
        .into_iter()
        .enumerate()
        .find(|(_, chars)| {
            chars.into_iter().cloned().collect::<BTreeSet<char>>().len() == arguments.n
        })
        .map(|(position, _)| position)
        .expect("valid message")
        + arguments.n
}
