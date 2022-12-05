#![feature(once_cell)]

use adventofcode2022::{Command, CommandResult};
use anyhow::Result;
use clap::Command as ClapCommand;
use std::cell::LazyCell;

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let commands: Vec<(&str, LazyCell<Box<dyn Command>>)> = vec![
        day01::DAY_01,
        day02::DAY_02,
        day03::DAY_03,
        day04::DAY_04,
        day05::DAY_05,
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
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommands(subcommands)
        .get_matches();

    commands
        .into_iter()
        .filter_map(|(name, command)| {
            matches
                .subcommand_matches(name)
                .map(|args| command.run(args))
        })
        .collect::<Result<Vec<CommandResult>>>()
        .map(|results| {
            results
                .into_iter()
                .for_each(|result| println!("{}", result))
        })
}
