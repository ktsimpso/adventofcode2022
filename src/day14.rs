use adventofcode2022::{flag_arg, parse_lines, parse_usize, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::ArgMatches;
use itertools::Itertools;
use std::{
    cell::LazyCell,
    cmp::{max, min},
    collections::{HashSet, VecDeque},
};

type ParseOutput = Vec<Vec<Line>>;

pub const DAY_14: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let bottom = flag_arg("bottom", 'b', "There is a bottom in the cave");
    let problem = Problem::new(
        "day14",
        "Find when sand reaches steady state.",
        "Path to the input file. Each line is a rock vien in the cave. Rock veins are continuous horizontal and vertical lines.",
        vec![bottom],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { bottom: false }, "Finds how much sand needs to fall before it falls into the abyss.")
    .with_part2(CommandLineArguments { bottom: true }, "Finds how much sand needs to fall before no more can fit");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    bottom: bool,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        bottom: *args.get_one::<bool>("bottom").expect("Valid arguments"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn get_adjacent_point(&self, direction: &Direction) -> Point {
        match direction {
            Direction::Down => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::DownLeft => Point {
                x: self.x - 1,
                y: self.y + 1,
            },
            Direction::DownRight => Point {
                x: self.x + 1,
                y: self.y + 1,
            },
        }
    }
}

#[derive(Debug, Clone)]
struct Line {
    start: Point,
    end: Point,
}

#[derive(Debug, Clone)]
enum Direction {
    Down,
    DownLeft,
    DownRight,
}

impl Line {
    fn get_min_y(&self) -> usize {
        min(self.start.y, self.end.y)
    }

    fn get_max_y(&self) -> usize {
        max(self.start.y, self.end.y)
    }

    fn get_min_x(&self) -> usize {
        min(self.start.x, self.end.x)
    }

    fn get_max_x(&self) -> usize {
        max(self.start.x, self.end.x)
    }

    fn to_points(self) -> HashSet<Point> {
        (self.get_min_x()..=self.get_max_x())
            .into_iter()
            .flat_map(|x| {
                (self.get_min_y()..=self.get_max_y())
                    .into_iter()
                    .map(move |y| Point { x, y })
            })
            .collect()
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_line_segments()).then_ignore(end())
}

fn parse_line_segments() -> impl Parser<char, Vec<Line>, Error = Simple<char>> {
    parse_point()
        .separated_by(just(" -> "))
        .at_least(2)
        .map(|points| {
            points
                .into_iter()
                .tuple_windows()
                .map(|(start, end)| Line {
                    start: start,
                    end: end,
                })
                .collect()
        })
}

fn parse_point() -> impl Parser<char, Point, Error = Simple<char>> {
    parse_usize()
        .then_ignore(just(","))
        .then(parse_usize())
        .map(|(x, y)| Point { x, y })
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let mut lines = input
        .into_iter()
        .flat_map(|lines| lines.into_iter())
        .collect::<Vec<Line>>();
    let mut max_y = lines.iter().map(|line| line.get_max_y()).max().unwrap_or(0);

    if arguments.bottom {
        max_y += 2;
        lines.push(Line {
            start: Point {
                x: 500 - max_y,
                y: max_y,
            },
            end: Point {
                x: 500 + max_y,
                y: max_y,
            },
        });
    }

    let mut count = 0usize;
    let mut occupied_points: HashSet<Point> =
        lines
            .into_iter()
            .map(|line| line.to_points())
            .fold(HashSet::new(), |mut acc, value| {
                acc.extend(value.into_iter());
                acc
            });
    let origin = Point { x: 500, y: 0 };
    let mut path = VecDeque::from([origin.clone()]);
    let valid_directions = vec![Direction::Down, Direction::DownLeft, Direction::DownRight];

    while let Some(mut current_point) = path.pop_back() {
        while current_point.y <= max_y {
            let next_point = valid_directions.iter().find_map(|direction| {
                is_valid_next_tile(&current_point, direction, &occupied_points)
            });

            match next_point {
                Some(point) => {
                    path.push_back(current_point);
                    current_point = point;
                }
                None => {
                    occupied_points.insert(current_point.clone());
                    count += 1;
                    break;
                }
            }
        }

        if current_point.y > max_y {
            break;
        }
    }
    count
}

fn is_valid_next_tile(
    point: &Point,
    direction: &Direction,
    sand_points: &HashSet<Point>,
) -> Option<Point> {
    let next_point = point.get_adjacent_point(direction);

    if sand_points.contains(&next_point) {
        None
    } else {
        Some(next_point)
    }
}
