use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, Problem,
};
use chumsky::{prelude::Simple, primitive::just, Parser};
use clap::ArgMatches;
use std::cell::LazyCell;

type ParseOutput = Vec<((usize, usize), (usize, usize))>;

#[derive(Debug, Clone)]
pub struct CommandLineArguments {}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        //n: *args.get_one::<usize>("number").expect("Valid arguments"),
    }
}

pub const DAY_04: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let problem = Problem::new(
        "day04",
        "Counts the number of elf paris which have overlapping work.",
        "Path to the input file. Each line is a comma serperated pair of work sections. Each work section start and end section is seperated by a -",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "Counts the number of elf pairs where one is fully overlapping");
    //.with_part2(CommandLineArguments {}, "part 2 help");
    Box::new(problem)
});

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_group())
}

fn parse_group() -> impl Parser<char, ((usize, usize), (usize, usize)), Error = Simple<char>> {
    parse_pair().then_ignore(just(',')).then(parse_pair())
}

fn parse_pair() -> impl Parser<char, (usize, usize), Error = Simple<char>> {
    parse_usize().then_ignore(just('-')).then(parse_usize())
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    input
        .into_iter()
        .filter(|((first_start, first_end), (second_start, second_end))| {
            (first_start <= second_start && first_end >= second_end)
                || (second_start <= first_start && second_end >= first_end)
        })
        .count()
}
