use adventofcode2022::{
    flag_arg, parse_isize, parse_lines, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::ArgMatches;
use std::{cell::LazyCell, collections::BTreeSet};

type ParseOutput = Vec<Operation>;

pub const DAY_10: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let cycles = single_arg(
        "cycles",
        'c',
        "The cycles to sample the signal strength from, comma delimited",
    )
    .value_delimiter(',')
    .value_terminator(";")
    .value_parser(clap::value_parser!(usize));
    let render = flag_arg("render", 'r', "Whether to render the screen or not");
    let problem = Problem::new(
        "day10",
        "Finds the signal strength at each of the target cycles then sums them. Optionally prints the result of the crt scan lines.",
        "Path to the input file. File should contain lines of either addx [usize] or noop to render a sprite to a crt",
        vec![cycles, render],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments {
            target_cycles: BTreeSet::from([20, 60, 100, 140, 180, 220]),
            render: false,
        },
        "Finds the signal strength for 20, 60, 100, 140, 180, and 220",
    )
    .with_part2(CommandLineArguments { target_cycles: BTreeSet::new(), render: true }, "Prints the crt with no signal strength");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    target_cycles: BTreeSet<usize>,
    render: bool,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        target_cycles: args
            .get_many::<usize>("cycles")
            .expect("Valid arguments")
            .cloned()
            .collect(),
        render: *args.get_one::<bool>("render").expect("Valid arguments"),
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Noop,
    Addx(isize),
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_operation()).then_ignore(end())
}

fn parse_operation() -> impl Parser<char, Operation, Error = Simple<char>> {
    let noop = just("noop").to(Operation::Noop);
    let addx = just("addx")
        .ignore_then(just(" "))
        .ignore_then(parse_isize())
        .map(|value| Operation::Addx(value));
    noop.or(addx)
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> isize {
    let mut x = 1isize;
    let mut cycle_count = 0usize;
    let mut sum = 0isize;

    input.into_iter().for_each(|operation| match operation {
        Operation::Noop => {
            update_cycle(&mut cycle_count, &mut sum, &x, &arguments);
        }
        Operation::Addx(value) => {
            update_cycle(&mut cycle_count, &mut sum, &x, &arguments);
            update_cycle(&mut cycle_count, &mut sum, &x, &arguments);
            x += value;
        }
    });

    sum
}

fn update_cycle(
    cycle_count: &mut usize,
    sum: &mut isize,
    x: &isize,
    arguments: &CommandLineArguments,
) {
    if arguments.render {
        print_cycle(&cycle_count, &x);
    }
    *cycle_count += 1;
    *sum += signal_strength_for_cycle(&cycle_count, &x, &arguments.target_cycles);
}

fn signal_strength_for_cycle(
    cycle_count: &usize,
    x: &isize,
    target_cycles: &BTreeSet<usize>,
) -> isize {
    target_cycles
        .get(cycle_count)
        .map(|target_cycle| *target_cycle as isize * x)
        .unwrap_or(0)
}

fn print_cycle(cycle_count: &usize, x: &isize) {
    let mod_cycle_count = (cycle_count % 40) as isize;
    let print_value =
        if *x == mod_cycle_count || x - 1 == mod_cycle_count || x + 1 == mod_cycle_count {
            "#"
        } else {
            "."
        };

    print!("{}", print_value);

    if mod_cycle_count == 39 {
        println!();
    }
}
