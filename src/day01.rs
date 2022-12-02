use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, Problem,
};
use chumsky::{prelude::Simple, Parser};
use clap::ArgMatches;
use std::cell::LazyCell;

type ParseOutput = Vec<Vec<usize>>;

pub const DAY_01: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let number = single_arg("number", 'n', "The number of elves to sum")
        .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day01",
        "Takes a list of elves backpacks calorie count and find the ones with the most",
        "Path to the input file. Input should be newline delimited groups integers. Each group represents one elf's bag, each line in the group is the caloric value of that item.",
    vec![number], parse_arguments, parse_file, run)
        .with_part1(CommandLineArguments { n: 1 }, "Finds the elf with the most calories in their bag and returns the sum of the calories")
        .with_part2(CommandLineArguments { n: 3 }, "Finds the elves with the 3 top most calories and sums the calories.");
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

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_between_blank_lines(parse_lines(parse_usize()))
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let mut bag_sums = input
        .into_iter()
        .map(|bag| bag.into_iter().sum())
        .collect::<Vec<usize>>();

    bag_sums.sort();
    bag_sums.reverse();

    bag_sums.into_iter().take(arguments.n).sum()
}
