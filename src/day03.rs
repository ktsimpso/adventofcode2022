use std::collections::BTreeSet;

use adventofcode2022::{parse_lines, Problem, ProblemWithTwoParts};
use bpaf::{construct, short, Parser as ArgParser};
use chumsky::{prelude::Simple, primitive::one_of, Parser};
use itertools::Itertools;

type ParseOutput = Vec<Vec<char>>;

pub const DAY_03: ProblemWithTwoParts<CommandLineArguments, ParseOutput, usize> = Problem::new(
    "day03",
    "Finds common items in elfen rucksacks and find their score.",
    "Path to the input file. File should contain one rucksack in each line. Rucksacks are represented by acii letters and are case sensitive.",
    parse_arguments,
    parse_file,
    run,
)
.with_part1(CommandLineArguments { split_sack: true, group_size: 1 }, "Split each rucksack in half and find the common item. Sum the common item's score.")
.with_part2(CommandLineArguments { split_sack: false, group_size: 3 }, "Find the common item in every 3 rucksacks. Sum the common item's score.");

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    split_sack: bool,
    group_size: usize,
}

fn parse_arguments() -> Box<dyn ArgParser<CommandLineArguments>> {
    let split_sack = short('s').long("split").flag(true, false);
    let group_size = short('g').long("group").argument::<usize>("INT");
    Box::new(construct!(CommandLineArguments {
        split_sack,
        group_size
    }))
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
        .map(|sack| {
            if arguments.split_sack {
                let (left, right) = sack.split_at(sack.len() / 2);
                let left_set: BTreeSet<char> = left.into_iter().cloned().collect();
                let right_set: BTreeSet<char> = right.into_iter().cloned().collect();
                left_set
                    .intersection(&right_set)
                    .into_iter()
                    .cloned()
                    .collect()
            } else {
                sack
            }
        })
        .chunks(arguments.group_size)
        .into_iter()
        .map(|group| {
            group
                .map(|sack| sack.into_iter().collect::<BTreeSet<char>>())
                .reduce(|acc, sack| acc.intersection(&sack).into_iter().cloned().collect())
                .expect("At least one sack")
                .into_iter()
                .map(|c| {
                    if c.is_uppercase() {
                        (c as usize - 'A' as usize) + 27
                    } else {
                        (c as usize - 'a' as usize) + 1
                    }
                })
                .into_iter()
                .sum::<usize>()
        })
        .sum()
}
