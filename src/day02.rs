use adventofcode2022::{
    parse_lines, parse_usize, Problem, ProblemWithOnePart, ProblemWithTwoParts,
};
use bpaf::{construct, short, Parser as ArgParser};
use chumsky::{
    prelude::Simple,
    primitive::{filter, just},
    Parser,
};

#[derive(Debug, Clone)]
pub enum Roshambo {
    Rock,
    Paper,
    Scissors,
}

type ParseOutput = Vec<(Roshambo, Roshambo)>;

pub const DAY_02: ProblemWithOnePart<Arguments, ParseOutput, usize> = Problem::new(
    "day02",
    "Parses and scores a secret strategy for a rock paper scissors tournament",
    "Path to the input file. Input should be lines with value A, B, or C followed by X, Y, or Z separated by a space. The first character represents the opponents move and the second is our strategy",
    parse_arguments,
    parse_file,
    run,
)
.with_part1(Arguments {}, "Our strategy is a certain value of Rock, Paper, or Scissors to use");

#[derive(Debug, Clone)]
pub struct Arguments {}

fn parse_arguments() -> Box<dyn ArgParser<Arguments>> {
    Box::new(construct!(Arguments {}))
}

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_game())
}

fn parse_game() -> impl Parser<char, (Roshambo, Roshambo), Error = Simple<char>> {
    parse_roshambo()
        .then_ignore(just(' '))
        .then(parse_roshambo())
}

fn parse_roshambo() -> impl Parser<char, Roshambo, Error = Simple<char>> {
    let rock = just('A').or(just('X')).to(Roshambo::Rock);
    let paper = just('B').or(just('Y')).to(Roshambo::Paper);
    let scissors = just('C').or(just('Z')).to(Roshambo::Scissors);

    rock.or(paper).or(scissors)
}

fn run(input: ParseOutput, arguments: Arguments) -> usize {
    let score = input
        .into_iter()
        .map(|(p1, p2)| match (p1, p2) {
            (Roshambo::Rock, Roshambo::Rock) => 3 + 1,
            (Roshambo::Rock, Roshambo::Paper) => 6 + 2,
            (Roshambo::Rock, Roshambo::Scissors) => 0 + 3,
            (Roshambo::Paper, Roshambo::Rock) => 0 + 1,
            (Roshambo::Paper, Roshambo::Paper) => 3 + 2,
            (Roshambo::Paper, Roshambo::Scissors) => 6 + 3,
            (Roshambo::Scissors, Roshambo::Rock) => 6 + 1,
            (Roshambo::Scissors, Roshambo::Paper) => 0 + 2,
            (Roshambo::Scissors, Roshambo::Scissors) => 3 + 3,
        })
        .sum();

    score
}
