use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, ParseError,
    PointDirection, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::ArgMatches;
use std::{
    cell::LazyCell,
    collections::{HashMap, HashSet, VecDeque},
    iter::once,
};

type ParseOutput = Vec<Vec<Tile>>;

pub const DAY_23: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let problem = Problem::new(
        "day23",
        "Finds the number of empty ground tiles after the elves have spread out for some iterations",
        "Path to the input file. The initial positions of the elves.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "The number of empty spaces after 10 rounds.");
    //.with_part2(CommandLineArguments { }, "part 2 help.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        //n: *args.get_one::<usize>("number").expect("Valid arguments"),
    }
}

#[derive(Debug, Clone)]
pub enum Tile {
    Elf,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: isize,
    y: isize,
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_tile().repeated().at_least(1)).then_ignore(end())
}

fn parse_tile() -> impl Parser<char, Tile, Error = Simple<char>> {
    let elf = just("#").to(Tile::Elf);
    let empty = just(".").to(Tile::Empty);
    elf.or(empty)
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let mut elf_points = input
        .into_iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.into_iter()
                .enumerate()
                .filter(|(_, tile)| match tile {
                    Tile::Elf => true,
                    Tile::Empty => false,
                })
                .map(move |(x, _)| Point {
                    x: x as isize,
                    y: y as isize,
                })
        })
        .collect::<HashSet<_>>();

    let mut directions = VecDeque::from([
        PointDirection::Up,
        PointDirection::Down,
        PointDirection::Left,
        PointDirection::Right,
    ]);

    let mut prev = elf_points.clone();
    let mut count = 0;

    while count < 10 {
        //print_elves(&elf_points);
        //println!();
        count += 1;

        elf_points = run_iteration(elf_points, &directions);
        let front = directions.pop_front().expect("Direction exists");
        directions.push_back(front);
    }

    //print_elves(&elf_points);

    let max_x = elf_points.iter().map(|point| point.x).max().unwrap_or(0);
    let min_x = elf_points.iter().map(|point| point.x).min().unwrap_or(0);
    let max_y = elf_points.iter().map(|point| point.y).max().unwrap_or(0);
    let min_y = elf_points.iter().map(|point| point.y).min().unwrap_or(0);

    (max_x - min_x + 1) as usize * (max_y - min_y + 1) as usize - elf_points.len()
}

fn run_iteration(
    elf_points: HashSet<Point>,
    move_order: &VecDeque<PointDirection>,
) -> HashSet<Point> {
    elf_points
        .iter()
        .map(|elf| {
            let north = Point {
                x: elf.x,
                y: elf.y - 1,
            };
            let north_east = Point {
                x: elf.x + 1,
                y: elf.y - 1,
            };
            let north_west = Point {
                x: elf.x - 1,
                y: elf.y - 1,
            };
            let south = Point {
                x: elf.x,
                y: elf.y + 1,
            };
            let south_east = Point {
                x: elf.x + 1,
                y: elf.y + 1,
            };
            let south_west = Point {
                x: elf.x - 1,
                y: elf.y + 1,
            };
            let west = Point {
                x: elf.x - 1,
                y: elf.y,
            };
            let east = Point {
                x: elf.x + 1,
                y: elf.y,
            };

            if [
                north, north_east, east, south_east, south, south_west, west, north_west,
            ]
            .into_iter()
            .all(|proposed_point| !elf_points.contains(&proposed_point))
            {
                return (*elf, *elf);
            }

            move_order
                .iter()
                .find_map(|direction| match direction {
                    PointDirection::Up => {
                        if [north, north_east, north_west]
                            .into_iter()
                            .all(|proposed_point| !elf_points.contains(&proposed_point))
                        {
                            Some((north, *elf))
                        } else {
                            None
                        }
                    }
                    PointDirection::Down => {
                        if [south, south_east, south_west]
                            .into_iter()
                            .all(|proposed_point| !elf_points.contains(&proposed_point))
                        {
                            Some((south, *elf))
                        } else {
                            None
                        }
                    }
                    PointDirection::Left => {
                        if [west, north_west, south_west]
                            .into_iter()
                            .all(|proposed_point| !elf_points.contains(&proposed_point))
                        {
                            Some((west, *elf))
                        } else {
                            None
                        }
                    }
                    PointDirection::Right => {
                        if [east, north_east, south_east]
                            .into_iter()
                            .all(|proposed_point| !elf_points.contains(&proposed_point))
                        {
                            Some((east, *elf))
                        } else {
                            None
                        }
                    }
                })
                .unwrap_or((*elf, *elf))
        })
        .fold(HashMap::new(), |mut acc, (proposed, elf)| {
            let points = acc.entry(proposed).or_insert(Vec::new());
            points.push(elf);
            acc
        })
        .into_iter()
        .flat_map(|(key, elves)| {
            if elves.len() == 1 {
                once(key).collect()
            } else {
                elves
            }
        })
        .collect()
}

fn print_elves(elves: &HashSet<Point>) {
    let max_x = elves.iter().map(|point| point.x).max().unwrap_or(0);
    let min_x = elves.iter().map(|point| point.x).min().unwrap_or(0);
    let max_y = elves.iter().map(|point| point.y).max().unwrap_or(0);
    let min_y = elves.iter().map(|point| point.y).min().unwrap_or(0);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if elves.contains(&Point { x, y }) {
                print!("#")
            } else {
                print!(".")
            }
        }
        println!();
    }
}
