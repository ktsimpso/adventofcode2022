use std::cell::LazyCell;

use adventofcode2022::{parse_lines, single_arg, Command, Problem};
use chumsky::{prelude::Simple, primitive::just, Parser};
use clap::{builder::EnumValueParser, ArgMatches, ValueEnum};

#[derive(Debug, Clone)]
pub enum Roshambo {
    Rock,
    Paper,
    Scissors,
}

impl Roshambo {
    fn get_score(&self) -> usize {
        match self {
            Roshambo::Rock => 1,
            Roshambo::Paper => 2,
            Roshambo::Scissors => 3,
        }
    }

    fn get_outcome(&self, opponents_move: &Roshambo) -> Outcome {
        match (opponents_move, self) {
            (Roshambo::Rock, Roshambo::Paper) => Outcome::Win,
            (Roshambo::Rock, Roshambo::Scissors) => Outcome::Lose,
            (Roshambo::Paper, Roshambo::Rock) => Outcome::Lose,
            (Roshambo::Paper, Roshambo::Scissors) => Outcome::Win,
            (Roshambo::Scissors, Roshambo::Rock) => Outcome::Win,
            (Roshambo::Scissors, Roshambo::Paper) => Outcome::Lose,
            _ => Outcome::Draw,
        }
    }
}

#[derive(Debug, Clone)]
enum Outcome {
    Win,
    Lose,
    Draw,
}

impl Outcome {
    fn get_score(&self) -> usize {
        match self {
            Outcome::Win => 6,
            Outcome::Lose => 0,
            Outcome::Draw => 3,
        }
    }

    fn get_roshambo(&self, opponents_move: &Roshambo) -> Roshambo {
        match (opponents_move, self) {
            (Roshambo::Rock, Outcome::Win) => Roshambo::Paper,
            (Roshambo::Rock, Outcome::Lose) => Roshambo::Scissors,
            (Roshambo::Paper, Outcome::Win) => Roshambo::Scissors,
            (Roshambo::Paper, Outcome::Lose) => Roshambo::Rock,
            (Roshambo::Scissors, Outcome::Win) => Roshambo::Rock,
            (Roshambo::Scissors, Outcome::Lose) => Roshambo::Paper,
            (a, Outcome::Draw) => a.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StrategyKey {
    X,
    Y,
    Z,
}

impl StrategyKey {
    fn to_roshambo(&self) -> Roshambo {
        match self {
            StrategyKey::X => Roshambo::Rock,
            StrategyKey::Y => Roshambo::Paper,
            StrategyKey::Z => Roshambo::Scissors,
        }
    }

    fn to_outcome(&self) -> Outcome {
        match self {
            StrategyKey::X => Outcome::Lose,
            StrategyKey::Y => Outcome::Draw,
            StrategyKey::Z => Outcome::Win,
        }
    }
}

#[derive(Debug, ValueEnum, Clone)]
pub enum Strategy {
    Roshambo,
    Outcome,
}

trait Round {
    fn get_round_result(&self) -> usize;
}

impl Round for (Roshambo, Roshambo) {
    fn get_round_result(&self) -> usize {
        self.1.get_outcome(&self.0).get_score() + self.1.get_score()
    }
}

impl Round for (Roshambo, Outcome) {
    fn get_round_result(&self) -> usize {
        self.1.get_roshambo(&self.0).get_score() + self.1.get_score()
    }
}

type ParseOutput = Vec<(Roshambo, StrategyKey)>;

pub const DAY_02: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let strategy = single_arg("strategy", 's', "The strategy to use in the game")
        .value_parser(EnumValueParser::<Strategy>::new());
    let problem = Problem::new(
        "day02",
        "Parses and scores a secret strategy for a rock paper scissors tournament",
        "Path to the input file. Input should be lines with value A, B, or C followed by X, Y, or Z separated by a space. The first character represents the opponents move and the second is our strategy",
    vec![strategy], parse_arguments, parse_file, run)
        .with_part1(CommandLineArguments { strategy: Strategy::Roshambo }, "Our strategy is a certain value of Rock, Paper, or Scissors to use")
        .with_part2(CommandLineArguments { strategy: Strategy::Outcome }, "Our strategy is a target outcome to have");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    strategy: Strategy,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    let strategy = args.get_one::<Strategy>("strategy").expect("Required arg");
    CommandLineArguments {
        strategy: strategy.clone(),
    }
}

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_game())
}

fn parse_game() -> impl Parser<char, (Roshambo, StrategyKey), Error = Simple<char>> {
    parse_roshambo()
        .then_ignore(just(' '))
        .then(parse_strategy())
}

fn parse_roshambo() -> impl Parser<char, Roshambo, Error = Simple<char>> {
    let rock = just('A').to(Roshambo::Rock);
    let paper = just('B').to(Roshambo::Paper);
    let scissors = just('C').to(Roshambo::Scissors);

    rock.or(paper).or(scissors)
}

fn parse_strategy() -> impl Parser<char, StrategyKey, Error = Simple<char>> {
    let lose = just('X').to(StrategyKey::X);
    let draw = just('Y').to(StrategyKey::Y);
    let win = just('Z').to(StrategyKey::Z);

    lose.or(draw).or(win)
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let score = input
        .into_iter()
        .map(|(opponents_move, strategy_key)| match arguments.strategy {
            Strategy::Roshambo => (opponents_move, strategy_key.to_roshambo()).get_round_result(),
            Strategy::Outcome => (opponents_move, strategy_key.to_outcome()).get_round_result(),
        })
        .sum();

    score
}
