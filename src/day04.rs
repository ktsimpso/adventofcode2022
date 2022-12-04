use adventofcode2022::{parse_lines, parse_usize, single_arg, Command, Problem};
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::{ArgMatches, ValueEnum};
use std::cell::LazyCell;

type ParseOutput = Vec<((usize, usize), (usize, usize))>;

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    overlap: OverlapCountStrategy,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OverlapCountStrategy {
    Full,
    Any,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        overlap: args
            .get_one::<OverlapCountStrategy>("overlap")
            .expect("Valid arguments")
            .clone(),
    }
}

pub const DAY_04: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let overlap = single_arg("overlap", 'o', "The overlap count strategy. Use Full to count only full overlapping work, or Any for partial overlapping")
        .value_parser(clap::value_parser!(OverlapCountStrategy));
    let problem = Problem::new(
        "day04",
        "Counts the number of elf paris which have overlapping work.",
        "Path to the input file. Each line is a comma serperated pair of work sections. Each work section start and end section is seperated by a -",
        vec![overlap],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { overlap: OverlapCountStrategy::Full }, "Counts the number of elf pairs where one is fully overlapping")
    .with_part2(CommandLineArguments { overlap: OverlapCountStrategy::Any }, "Counts the number of elf pairs with any overlapping");
    Box::new(problem)
});

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_group()).then_ignore(end())
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
        .filter(
            |((first_start, first_end), (second_start, second_end))| match arguments.overlap {
                OverlapCountStrategy::Full => {
                    (first_start <= second_start && first_end >= second_end)
                        || (second_start <= first_start && second_end >= first_end)
                }
                OverlapCountStrategy::Any => {
                    (first_start < second_start && second_start <= first_end)
                        || (second_start < first_start && first_start <= second_end)
                        || first_start == second_start
                }
            },
        )
        .count()
}
