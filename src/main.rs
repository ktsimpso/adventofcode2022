use adventofcode2022::{Command, CommandParser};
use bpaf::Parser as ArgParser;

mod day01;
mod day02;
mod day03;

fn main() {
    let day_01 = day01::DAY_01.command();
    let day_02 = day02::DAY_02.command();
    let day_03 = day03::DAY_03.command();
    let value = day_03.to_options().run();

    println!("{}", value.run());
}
