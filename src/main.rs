#![feature(lazy_cell)]

use adventofcode2022::{Command, CommandResult};
use anyhow::Result;
use clap::Command as ClapCommand;
use std::{
    cell::LazyCell,
    time::{Duration, Instant},
};

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day17;
mod day18;
mod day19;
mod day20;
mod day21;
mod day22;
mod day23;
mod day24;
mod day25;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let commands: Vec<(&str, LazyCell<Box<dyn Command>>)> = vec![
        day01::DAY_01,
        day02::DAY_02,
        day03::DAY_03,
        day04::DAY_04,
        day05::DAY_05,
        day06::DAY_06,
        day07::DAY_07,
        day08::DAY_08,
        day09::DAY_09,
        day10::DAY_10,
        day11::DAY_11,
        day12::DAY_12,
        day13::DAY_13,
        day14::DAY_14,
        day15::DAY_15,
        day16::DAY_16,
        day17::DAY_17,
        day18::DAY_18,
        day19::DAY_19,
        day20::DAY_20,
        day21::DAY_21,
        day22::DAY_22,
        day23::DAY_23,
        day24::DAY_24,
        day25::DAY_25,
    ]
    .into_iter()
    .map(|command| (command.get_name(), command))
    .collect();
    let subcommands = commands
        .iter()
        .map(|(_, command)| command.get_subcommand())
        .collect::<Vec<_>>();

    let matches = ClapCommand::new("Advent of Code 2022")
        .version(VERSION)
        .about("Run the advent of code problems from this main program")
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommands(subcommands)
        .get_matches();

    commands
        .into_iter()
        .filter_map(|(name, command)| {
            matches.subcommand_matches(name).map(|args| {
                println!("=============Running {:}=============", command.get_name());
                let now = Instant::now();
                let result = command.run(args);
                let elapsed = now.elapsed();
                result.map(|r| (r, elapsed))
            })
        })
        .collect::<Result<Vec<(CommandResult, Duration)>>>()
        .map(|results| {
            results.into_iter().for_each(|(result, elapsed)| {
                println!("{}", result);
                println!("Took {:#?} to run", elapsed)
            })
        })
}
