use adventofcode2022::{parse_line, parse_usize, Problem, ProblemWithTwoParts};
use bpaf::{construct, short, Parser as ArgParser};
use chumsky::{prelude::Simple, Parser};

pub const DAY_01: ProblemWithTwoParts<Arguments1, Vec<usize>, usize> = Problem::new(
    "day01",
    "about the problem",
    "Path to the input file. Input should be newline delimited integers.",
    parse_arguments,
    parse_file,
    run,
)
.with_part1(Arguments1 { s: 1 }, "Docs for part1")
.with_part2(Arguments1 { s: 3 }, "Docs for part2");

#[derive(Debug, Clone)]
pub struct Arguments1 {
    s: usize,
}

fn parse_arguments() -> Box<dyn ArgParser<Arguments1>> {
    let s = short('s').help("Test argument").argument::<usize>("SHORT");
    Box::new(construct!(Arguments1 { s }))
}

fn parse_file(file: String) -> Vec<usize> {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, Vec<usize>, Error = Simple<char>> {
    parse_line(parse_usize())
}

fn run(input: Vec<usize>, arguments: Arguments1) -> usize {
    0
}
