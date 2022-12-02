#![feature(once_cell)]

use std::cell::LazyCell;

use adventofcode2022::Command;
use clap::Command as ClapCommand;

mod day01;
mod day02;
mod day03;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let commands: Vec<(&str, LazyCell<Box<dyn Command>>)> =
        vec![day01::DAY_01, day02::DAY_02, day03::DAY_03]
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
        .for_each(|result| println!("{}", result))
}
