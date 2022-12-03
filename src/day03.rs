use std::collections::BTreeSet;

use adventofcode2022::{parse_lines, Problem, ProblemWithOnePart};
use bpaf::{construct, short, Parser as ArgParser};
use chumsky::{
    prelude::Simple,
    primitive::{one_of, TakeUntil},
    Parser,
};

type ParseOutput = Vec<Vec<char>>;

pub const DAY_03: ProblemWithOnePart<CommandLineArguments, ParseOutput, usize> = Problem::new(
    "day03",
    "Finds common items in evlen rucksacks and find their score",
    "Path to the input file. File should contain one rucksack in each line. Rucksacks are represented by acii letters and are case sensitive",
    parse_arguments,
    parse_file,
    run,
)
.with_part1(CommandLineArguments {}, "Split each rucksack in half and find the common item. Sum the common item's score");
//.with_part2(CommandLineArguments { }, "part 2 about");

#[derive(Debug, Clone)]
pub struct CommandLineArguments {}

fn parse_arguments() -> Box<dyn ArgParser<CommandLineArguments>> {
    Box::new(construct!(CommandLineArguments {}))
}

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    let mut upper = ('A'..='Z').collect::<String>();
    let lower = ('a'..='z').collect::<String>();
    upper.push_str(&lower);
    parse_lines(one_of(upper).repeated())
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    input
        .into_iter()
        .map(|items| {
            let (left, right) = items.split_at(items.len() / 2);
            let left_set: BTreeSet<char> = left.into_iter().cloned().collect();
            let right_set: BTreeSet<char> = right.into_iter().cloned().collect();
            left_set
                .intersection(&right_set)
                .map(|c| {
                    if c.is_uppercase() {
                        (*c as usize - 'A' as usize) + 27
                    } else {
                        (*c as usize - 'a' as usize) + 1
                    }
                })
                .into_iter()
                .sum::<usize>()
        })
        .sum()
}
