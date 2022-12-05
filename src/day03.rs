use std::{cell::LazyCell, collections::BTreeSet};

use adventofcode2022::{flag_arg, parse_lines, single_arg, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, one_of},
    Parser,
};
use clap::ArgMatches;
use itertools::Itertools;

type ParseOutput = Vec<Vec<char>>;

pub const DAY_03: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let split = flag_arg("split", 's', "Splits each rucksack in half");
    let group_size = single_arg(
        "group",
        'g',
        "How many rucks sacks should be included to check for common items.",
    )
    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day03",
        "Finds common items in elfen rucksacks and find their score.",
        "Path to the input file. File should contain one rucksack in each line. Rucksacks are represented by acii letters and are case sensitive.",
    vec![split, group_size], parse_arguments, parse_file, run)
        .with_part1(CommandLineArguments { split_sack: true, group_size: 1 }, "Split each rucksack in half and find the common item. Sum the common item's score.")
        .with_part2(CommandLineArguments { split_sack: false, group_size: 3 }, "Find the common item in every 3 rucksacks. Sum the common item's score.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    split_sack: bool,
    group_size: usize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    let split_sack = *args.get_one::<bool>("split").expect("Required arg");
    let group_size = *args.get_one::<usize>("group").expect("Required arg");
    CommandLineArguments {
        split_sack,
        group_size,
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    let mut upper = ('A'..='Z').collect::<String>();
    let lower = ('a'..='z').collect::<String>();
    upper.push_str(&lower);
    parse_lines(one_of(upper).repeated()).then_ignore(end())
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
