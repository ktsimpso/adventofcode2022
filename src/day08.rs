use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{prelude::Simple, primitive::end, primitive::one_of, primitive::todo, text, Parser};
use clap::ArgMatches;
use itertools::Itertools;
use std::cell::LazyCell;

type ParseOutput = Vec<Vec<usize>>;

pub const DAY_08: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day08",
        "Servey's a forest and gives stats about the visibility of trees in the forest",
        "Path to the input file. Should consist of lines of and equal number of integers between 0-9",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "Counts the number of trees that are visible from the edge of the forest.");
    //.with_part2(CommandLineArguments {}, "part 2 help");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    //n: usize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        //n: *args.get_one::<usize>("number").expect("Valid arguments"),
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    let digits = ('0'..='9').into_iter().collect::<String>();

    parse_lines(
        one_of(digits)
            .try_map(|value: char, span| {
                parse_usize()
                    .parse(value.to_string())
                    .map_err(|op| Simple::custom(span, op.into_iter().join("\n")))
            })
            .repeated()
            .at_least(1),
    )
    .then_ignore(end())
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    input
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(x, _)| is_visible(*x, y, &input))
                .count()
        })
        .sum()
}

fn is_visible(x: usize, y: usize, forest: &Vec<Vec<usize>>) -> bool {
    is_visible_left(x, y, forest)
        || is_visible_right(x, y, forest)
        || is_visible_down(x, y, forest)
        || is_visible_up(x, y, forest)
}

fn is_visible_left(x: usize, y: usize, forest: &Vec<Vec<usize>>) -> bool {
    let mut current = x;
    let mut visible = true;
    let height = forest.get(y).expect("valid y").get(x).expect("valid x");

    while current > 0 {
        current -= 1;
        let other = forest
            .get(y)
            .expect("valid y")
            .get(current)
            .expect("valid x");
        if other >= height {
            visible = false;
            break;
        }
    }

    visible
}

fn is_visible_right(x: usize, y: usize, forest: &Vec<Vec<usize>>) -> bool {
    let mut current = x;
    let mut visible = true;
    let height = forest.get(y).expect("valid y").get(x).expect("valid x");

    while current < forest.get(0).expect("At least 1 row").len() - 1 {
        current += 1;
        let other = forest
            .get(y)
            .expect("valid y")
            .get(current)
            .expect("valid x");
        if other >= height {
            visible = false;
            break;
        }
    }

    visible
}

fn is_visible_up(x: usize, y: usize, forest: &Vec<Vec<usize>>) -> bool {
    let mut current = y;
    let mut visible = true;
    let height = forest.get(y).expect("valid y").get(x).expect("valid x");

    while current > 0 {
        current -= 1;
        let other = forest
            .get(current)
            .expect("valid y")
            .get(x)
            .expect("valid x");
        if other >= height {
            visible = false;
            break;
        }
    }

    visible
}

fn is_visible_down(x: usize, y: usize, forest: &Vec<Vec<usize>>) -> bool {
    let mut current = y;
    let mut visible = true;
    let height = forest.get(y).expect("valid y").get(x).expect("valid x");

    while current < forest.len() - 1 {
        current += 1;
        let other = forest
            .get(current)
            .expect("valid y")
            .get(x)
            .expect("valid x");
        if other >= height {
            visible = false;
            break;
        }
    }

    visible
}
