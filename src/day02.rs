use adventofcode2022::{parse_lines, parse_usize, Problem, ProblemWithTwoParts};
use bpaf::{construct, short, Parser as ArgParser};
use chumsky::{prelude::Simple, primitive::filter, Parser};

pub const DAY_02: Problem<Arguments1, Vec<char>, usize> = Problem::new(
    "day02",
    "about the problem",
    "Path to the input file. Input should be newline delimited chars.",
    parse_arguments,
    parse_file,
    run,
);

#[derive(Debug, Clone)]
pub struct Arguments1 {
    s: String,
    b: char,
}

fn parse_arguments() -> Box<dyn ArgParser<Arguments1>> {
    let s = short('s').help("Test argument").argument::<String>("SHORT");
    let b = short('b').help("Test argument 2").argument::<char>("SHORT");
    Box::new(construct!(Arguments1 { s, b }))
}

fn parse_file(file: String) -> Vec<char> {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, Vec<char>, Error = Simple<char>> {
    parse_lines(filter(|c| true))
}

fn run(input: Vec<char>, arguments: Arguments1) -> usize {
    0
}
