use adventofcode2022::{
    parse_between_blank_lines, parse_usize, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    recursive::recursive,
    text, Parser,
};
use clap::{ArgMatches, ValueEnum};
use std::{cell::LazyCell, iter::once};

type ParseOutput = Vec<(Signal, Signal)>;

pub const DAY_13: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let ordering = single_arg("ordering", 'o', "The ordering strategy to use.")
        .value_parser(clap::value_parser!(PacketOrder));
    let problem = Problem::new(
        "day13",
        "Determines order properties of packets",
        "Path to the input file. Groups of two packets, one packet on each line. Each group is seperated by a newline.",
        vec![ordering],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { ordering: PacketOrder::Local }, "Finds the 1 based index of the packets who's pair are in order and sums them.")
    .with_part2(CommandLineArguments { ordering: PacketOrder::Global }, "Orders all packets and returns the product of the 1 based indexes fro the divider packets.");
    Box::new(problem)
});

#[derive(Debug, Clone, ValueEnum)]
pub enum PacketOrder {
    Local,
    Global,
}

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    ordering: PacketOrder,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        ordering: args
            .get_one::<PacketOrder>("ordering")
            .expect("Valid arguments")
            .clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    Literal(usize),
    List(Vec<Signal>),
}

impl PartialOrd for Signal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        compare_signals(self, other).map(|result| {
            if result {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        })
    }
}

impl Ord for Signal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_between_blank_lines(parse_signal_pair()).then_ignore(end())
}

fn parse_signal_pair() -> impl Parser<char, (Signal, Signal), Error = Simple<char>> {
    parse_signal()
        .then_ignore(text::newline())
        .then(parse_signal())
}

fn parse_signal() -> impl Parser<char, Signal, Error = Simple<char>> {
    recursive(|signal| {
        let list = signal
            .separated_by(just(","))
            .delimited_by(just('['), just(']'))
            .map(|signals| Signal::List(signals));

        parse_usize().map(|value| Signal::Literal(value)).or(list)
    })
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    match arguments.ordering {
        PacketOrder::Local => input
            .into_iter()
            .enumerate()
            .filter(|(_, (left, right))| compare_signals(left, right).unwrap_or(false))
            .map(|(index, _)| index + 1)
            .sum(),
        PacketOrder::Global => {
            let divider1 = Signal::List(vec![Signal::List(vec![Signal::Literal(2)])]);
            let divider2 = Signal::List(vec![Signal::List(vec![Signal::Literal(6)])]);
            let mut flat_signals = input
                .into_iter()
                .flat_map(|signals| once(signals.0).chain(once(signals.1)))
                .chain(once(divider1.clone()))
                .chain(once(divider2.clone()))
                .collect::<Vec<_>>();
            flat_signals.sort();

            flat_signals
                .into_iter()
                .enumerate()
                .filter(|(_, signal)| signal == &divider1 || signal == &divider2)
                .map(|(index, _)| index + 1)
                .product()
        }
    }
}

fn compare_signals(left: &Signal, right: &Signal) -> Option<bool> {
    match (left, right) {
        (Signal::Literal(left_value), Signal::Literal(right_value)) => {
            if left_value < right_value {
                Some(true)
            } else if left_value == right_value {
                None
            } else {
                Some(false)
            }
        }
        (Signal::Literal(_), Signal::List(_)) => {
            compare_signals(&Signal::List(vec![left.clone()]), right)
        }
        (Signal::List(_), Signal::Literal(_)) => {
            compare_signals(left, &Signal::List(vec![right.clone()]))
        }
        (Signal::List(left_values), Signal::List(right_values)) => {
            let result = left_values
                .iter()
                .zip(right_values.iter())
                .find_map(|(left_value, right_value)| compare_signals(left_value, right_value));
            match result {
                Some(_) => result,
                None => {
                    if left_values.len() < right_values.len() {
                        Some(true)
                    } else if left_values.len() == right_values.len() {
                        None
                    } else {
                        Some(false)
                    }
                }
            }
        }
    }
}
