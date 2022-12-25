use adventofcode2022::{parse_lines, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::ArgMatches;
use itertools::Itertools;
use std::{cell::LazyCell, fmt::Display};

type ParseOutput = Vec<SnafuNumber>;

pub const DAY_25: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let problem = Problem::new(
        "day25",
        "Sums snafu numbers",
        "Path to the input file. The snafu numbers to sum, one snafu number on each line.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments {},
        "Sums the snafu numbers for the default input.",
    );
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {}

fn parse_arguments(_args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {}
}

#[derive(Debug, Clone)]
pub enum Snafu {
    Two,
    One,
    Zero,
    Minus,
    DoubleMinus,
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(
        parse_snafu()
            .repeated()
            .at_least(1)
            .map(|number| SnafuNumber(number)),
    )
    .then_ignore(end())
}

fn parse_snafu() -> impl Parser<char, Snafu, Error = Simple<char>> {
    let two = just('2').to(Snafu::Two);
    let one = just('1').to(Snafu::One);
    let zero = just('0').to(Snafu::Zero);
    let minus = just('-').to(Snafu::Minus);
    let double_minus = just('=').to(Snafu::DoubleMinus);

    two.or(one).or(zero).or(minus).or(double_minus)
}

fn run(input: ParseOutput, _arguments: CommandLineArguments) -> String {
    let result: SnafuNumber = input
        .into_iter()
        .map(|snafu| {
            let integer: isize = snafu.into();
            integer
        })
        .sum::<isize>()
        .into();

    result.to_string()
}

struct SnafuNumber(Vec<Snafu>);

impl Display for SnafuNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = self
            .0
            .iter()
            .map(|value| match value {
                Snafu::Two => '2',
                Snafu::One => '1',
                Snafu::Zero => '0',
                Snafu::Minus => '-',
                Snafu::DoubleMinus => '=',
            })
            .join("");

        write!(f, "{}", result)
    }
}

impl Into<isize> for SnafuNumber {
    fn into(self) -> isize {
        let length = self.0.len();
        self.0
            .iter()
            .enumerate()
            .map(|(index, value)| {
                5_isize.pow((length - index - 1) as u32)
                    * match value {
                        Snafu::Two => 2,
                        Snafu::One => 1,
                        Snafu::Zero => 0,
                        Snafu::Minus => -1,
                        Snafu::DoubleMinus => -2,
                    }
            })
            .sum::<isize>()
    }
}

impl From<isize> for SnafuNumber {
    fn from(value: isize) -> Self {
        let mut number = value;
        let mut snafu = Vec::new();
        let mut carry = 0;
        while number > 0 {
            let remainder = number % 5 + carry;
            number = number / 5;

            if number == 0 && remainder == 0 {
                break;
            }

            match remainder {
                1 => {
                    snafu.push(Snafu::One);
                    carry = 0;
                }
                2 => {
                    snafu.push(Snafu::Two);
                    carry = 0;
                }
                3 => {
                    snafu.push(Snafu::DoubleMinus);
                    carry = 1;
                }
                4 => {
                    snafu.push(Snafu::Minus);
                    carry = 1;
                }
                5 => {
                    snafu.push(Snafu::Zero);
                    carry = 1;
                }
                _ => {
                    snafu.push(Snafu::Zero);
                    carry = 0;
                }
            }
        }

        snafu.reverse();

        SnafuNumber(snafu)
    }
}
