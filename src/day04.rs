use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, Problem,
};
use chumsky::{prelude::Simple, Parser};
use clap::ArgMatches;
use std::cell::LazyCell;

type ParseOutput = Vec<usize>;

pub const DAY_04: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let problem = Problem::new(
        "day04",
        "day 4 help ",
        "Path to the input file. file help",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "part 1 help")
    .with_part2(CommandLineArguments {}, "part 2 help");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        //n: *args.get_one::<usize>("number").expect("Valid arguments"),
    }
}

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_usize())
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    0
}
