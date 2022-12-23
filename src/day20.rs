use adventofcode2022::{parse_isize, parse_lines, single_arg, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{prelude::Simple, primitive::end, Parser};
use clap::ArgMatches;
use std::{cell::LazyCell, fmt::Debug};

type ParseOutput = Vec<isize>;

pub const DAY_20: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let iterations = single_arg("iterations", 'i', "The number of times to remix the file")
        .value_parser(clap::value_parser!(usize));
    let decryption_key = single_arg("key", 'k', "The key to decrypt the file.")
        .value_parser(clap::value_parser!(isize));
    let problem = Problem::new(
        "day20",
        "Finds the grove coordinates based on the encrypted file and sums them.",
        "Path to the input file. A newline delimited list of numbers that are mixed up.",
        vec![iterations, decryption_key],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments {
            iterations: 1,
            decryption_key: 1,
        },
        "Remixes the file once with a 1 key and returns the result.",
    )
    .with_part2(
        CommandLineArguments {
            iterations: 10,
            decryption_key: 811_589_153,
        },
        "Remixes the file 10 times with a decryption key of 811_589_153",
    );
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    iterations: usize,
    decryption_key: isize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        iterations: *args
            .get_one::<usize>("iterations")
            .expect("Valid arguments"),
        decryption_key: *args.get_one::<isize>("key").expect("Valid arguments"),
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_isize()).then_ignore(end())
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> isize {
    let decryption_key = arguments.decryption_key;
    let input = input
        .into_iter()
        .map(|value| value * decryption_key)
        .collect::<Vec<_>>();

    let max_index = input.len();
    let max_isize_index = max_index as isize;
    let mut indexes = input
        .iter()
        .enumerate()
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    for _ in 0..arguments.iterations {
        input.iter().enumerate().for_each(|(index, value)| {
            let value = value;
            let old = indexes.iter().position(|i| &index == i).expect("exists");
            indexes.remove(old);
            let new = (old as isize + value).rem_euclid(max_isize_index - 1);
            indexes.insert(new as usize, index);
        });
    }

    let result = indexes
        .into_iter()
        .filter_map(|index| input.get(index))
        .collect::<Vec<_>>();

    let zero_index = result
        .iter()
        .position(|value| value == &&0)
        .expect("0 exists");

    let first = result.get((zero_index + 1000) % max_index).expect("Exists");
    let second = result.get((zero_index + 2000) % max_index).expect("Exists");
    let third = result.get((zero_index + 3000) % max_index).expect("Exists");

    *first + *second + *third
}
