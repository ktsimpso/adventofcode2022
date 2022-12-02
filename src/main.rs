use adventofcode2022::{Command, CommandParser};
use bpaf::{construct, Parser as ArgParser};

mod day01;
mod day02;

fn main() {
    let day_01 = day01::DAY_01.command();
    let day_02 = day02::DAY_02.command();
    let value = day_02.to_options().run();

    println!("{}", value.run());
}
