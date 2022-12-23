use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, BoundedPoint, Command,
    ParseError, PointDirection, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    text, Parser,
};
use clap::ArgMatches;
use std::cell::LazyCell;

type ParseOutput = (Vec<Vec<Tile>>, Vec<Instruction>);

pub const DAY_22: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day22",
        "Traverses the path in a grid and find the final position and facing.",
        "Path to the input file. The grid where a . is an empty space, and a # is a wall. Voids are permitted.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "Finds the final position on the grid.");
    //.with_part2(CommandLineArguments { }, "part 2 help");
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tile {
    Nothing,
    Space,
    Wall,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    RotateClockwise,
    RotateCounterClockwise,
    Distance(usize),
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_tiles()
        .then_ignore(text::newline())
        .then(parse_instruction().repeated().at_least(1))
        .then_ignore(text::newline())
        .then_ignore(end())
}

fn parse_instruction() -> impl Parser<char, Instruction, Error = Simple<char>> {
    let distance = parse_usize().map(|value| Instruction::Distance(value));
    let clockwise = just("R").to(Instruction::RotateClockwise);
    let counter_clockwise = just("L").to(Instruction::RotateCounterClockwise);

    distance.or(clockwise).or(counter_clockwise)
}

fn parse_tiles() -> impl Parser<char, Vec<Vec<Tile>>, Error = Simple<char>> {
    parse_lines(parse_tile().repeated().at_least(1))
}

fn parse_tile() -> impl Parser<char, Tile, Error = Simple<char>> {
    let nothing = just(" ").to(Tile::Nothing);
    let space = just(".").to(Tile::Space);
    let wall = just("#").to(Tile::Wall);

    nothing.or(space).or(wall)
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let max_x = input.0.iter().map(|row| row.len()).max().unwrap_or(0);
    let max_y = input.0.len();

    let board = input
        .0
        .into_iter()
        .map(|mut row| {
            row.extend(vec![Tile::Nothing; max_x - row.len()].iter().cloned());
            row
        })
        .collect::<Vec<_>>();

    let (point, direction) = traverse_grid(&board, &input.1, max_x - 1, max_y - 1);

    (point.y + 1) * 1000
        + (point.x + 1) * 4
        + match direction {
            PointDirection::Up => 3,
            PointDirection::Down => 1,
            PointDirection::Left => 2,
            PointDirection::Right => 0,
        }
}

fn traverse_grid(
    board: &Vec<Vec<Tile>>,
    instructions: &Vec<Instruction>,
    max_x: usize,
    max_y: usize,
) -> (BoundedPoint, PointDirection) {
    let mut current_point = board
        .get(0)
        .and_then(|row| {
            row.iter().enumerate().find(|(x, tile)| match tile {
                Tile::Space => true,
                _ => false,
            })
        })
        .map(|(x, _)| BoundedPoint {
            x,
            y: 0,
            max_x,
            max_y,
        })
        .expect("Start exists");
    let mut current_direction = PointDirection::Right;

    instructions
        .iter()
        .for_each(|instruction| match instruction {
            Instruction::RotateClockwise => {
                current_direction = match current_direction {
                    PointDirection::Up => PointDirection::Right,
                    PointDirection::Down => PointDirection::Left,
                    PointDirection::Left => PointDirection::Up,
                    PointDirection::Right => PointDirection::Down,
                }
            }
            Instruction::RotateCounterClockwise => {
                current_direction = match current_direction {
                    PointDirection::Up => PointDirection::Left,
                    PointDirection::Down => PointDirection::Right,
                    PointDirection::Left => PointDirection::Down,
                    PointDirection::Right => PointDirection::Up,
                }
            }
            Instruction::Distance(value) => {
                for _ in 0..*value {
                    let mut next_point = current_point.get_adjacent_wrapping(&current_direction);
                    let mut next_tile = board
                        .get(next_point.y)
                        .and_then(|row| row.get(next_point.x))
                        .expect("Tile Exists");
                    while next_tile == &Tile::Nothing {
                        next_point = next_point.get_adjacent_wrapping(&current_direction);
                        next_tile = board
                            .get(next_point.y)
                            .and_then(|row| row.get(next_point.x))
                            .expect("Tile Exists");
                    }

                    if next_tile == &Tile::Wall {
                        break;
                    }

                    current_point = next_point;
                }
            }
        });

    (current_point, current_direction)
}
