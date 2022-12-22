use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, BoundedPoint, Command,
    ParseError, PointDirection, Problem,
};
use anyhow::Result;
use chumsky::{chain::Chain, prelude::Simple, primitive::end, primitive::just, text, Parser};
use clap::ArgMatches;
use itertools::Itertools;
use std::{
    cell::LazyCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
};

type ParseOutput = Vec<PointDirection>;

pub const DAY_17: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day17",
        "Finds the height of falling rocks after a number of rocks have fallen",
        "Path to the input file. The wind direction at any given iteration. Cycles to the start once input ends.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "Finds the height of the rock tower after 2022 iterations.");
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

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_direction()
        .repeated()
        .then_ignore(text::newline())
        .then_ignore(end())
}

fn parse_direction() -> impl Parser<char, PointDirection, Error = Simple<char>> {
    let right = just(">").to(PointDirection::Right);
    let left = just("<").to(PointDirection::Left);
    right.or(left)
}

type Cave = VecDeque<Vec<Option<RockType>>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Rock(Vec<Vec<Option<()>>>);

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
enum RockType {
    Active,
    Solid,
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let horizontal = Rock(vec![vec![Some(()), Some(()), Some(()), Some(())]]);
    let plus = Rock(vec![
        vec![None, Some(()), None],
        vec![Some(()), Some(()), Some(())],
        vec![None, Some(()), None],
    ]);
    let chair = Rock(vec![
        vec![None, None, Some(())],
        vec![None, None, Some(())],
        vec![Some(()), Some(()), Some(())],
    ]);
    let vertical = Rock(vec![
        vec![Some(())],
        vec![Some(())],
        vec![Some(())],
        vec![Some(())],
    ]);
    let square = Rock(vec![vec![Some(()), Some(())], vec![Some(()), Some(())]]);

    let mut rocks = [horizontal.clone(), plus, chair, vertical, square]
        .into_iter()
        .cycle();
    let mut wind = input.into_iter().cycle();

    let mut cave: Cave = VecDeque::from(vec![vec![None, None, None, None, None, None, None]]);
    let mut count = 0;

    for i in 0..2022usize {
        let next_rock = rocks.next().expect("Rock exists");
        cave = add_rock(cave, next_rock.clone());

        loop {
            let wind_direction = wind.next().expect("window blows");
            if can_move_rock(&cave, &wind_direction) {
                cave = move_rock(cave, wind_direction)
            }

            if can_move_rock(&cave, &PointDirection::Up) {
                cave = move_rock(cave, PointDirection::Up)
            } else {
                cave = freeze_rock(cave);
                break;
            }
        }

        if i % 50 == 0 {
            let check_split_points = (0..7)
                .into_iter()
                .map(|index| get_tallest_rock_from_cave_at_index(&cave, index))
                .collect::<Vec<_>>();
            if check_split_points.iter().all(|point| point > &0) {
                let split_point = check_split_points.into_iter().min().expect("at least one");
                count += split_point - 1;
                let new = cave.split_off(split_point - 1);
                let old = cave;
                cave = new;
            }
        }
    }
    get_tallest_rock_from_cave(&cave) + count
}

fn freeze_rock(mut cave: Cave) -> Cave {
    cave.iter_mut().for_each(|row| {
        row.iter_mut()
            .filter(|space| match space {
                Some(RockType::Active) => true,
                _ => false,
            })
            .for_each(|space| *space = Some(RockType::Solid))
    });
    cave
}

fn move_rock(mut cave: Cave, direction: PointDirection) -> Cave {
    let max_y = cave.len() - 1;
    let max_x = 6;
    let active_points = cave
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(x, space)| match space {
                    Some(RockType::Active) => Some(BoundedPoint { x, y, max_x, max_y }),
                    _ => None,
                })
        })
        .collect::<Vec<_>>();

    active_points.iter().for_each(|point| {
        let space = cave
            .get_mut(point.y)
            .and_then(|row| row.get_mut(point.x))
            .expect("Point exists");
        *space = None;
    });

    active_points
        .iter()
        .filter_map(|point| point.get_adjacent(&direction))
        .for_each(|point| {
            let space = cave
                .get_mut(point.y)
                .and_then(|row| row.get_mut(point.x))
                .expect("Point exists");
            *space = Some(RockType::Active);
        });
    cave
}

fn can_move_rock(cave: &Cave, direction: &PointDirection) -> bool {
    let max_y = cave.len() - 1;
    let max_x = 6;
    let active_points = cave.iter().enumerate().flat_map(|(y, row)| {
        row.iter()
            .enumerate()
            .filter_map(move |(x, space)| match space {
                Some(RockType::Active) => Some(BoundedPoint { x, y, max_x, max_y }),
                _ => None,
            })
    });
    active_points.into_iter().all(|point| {
        point
            .get_adjacent(direction)
            .filter(|new_point| {
                cave.get(new_point.y)
                    .and_then(|row| row.get(new_point.x))
                    .filter(|next_space| match next_space {
                        Some(RockType::Active) => true,
                        Some(RockType::Solid) => false,
                        None => true,
                    })
                    .is_some()
            })
            .is_some()
    })
}

fn add_rock(mut cave: Cave, rock: Rock) -> Cave {
    cave = add_height_padding(cave);
    for y in (0..rock.0.len()).rev() {
        let row = rock
            .0
            .get(y)
            .expect("Row exists")
            .into_iter()
            .map(|item| match item {
                Some(_) => Some(RockType::Active),
                None => None,
            });
        let mut new_row = vec![None, None];
        new_row.extend(row);
        if new_row.len() < 7 {
            let padding = 7 - new_row.len();
            new_row.extend((0..padding).into_iter().map(|_| None))
        }
        cave.push_back(new_row);
    }
    cave
}

fn get_tallest_rock_from_cave(cave: &Cave) -> usize {
    cave.iter()
        .enumerate()
        .rev()
        .find(|(_, row)| {
            row.iter().any(|space| match space {
                Some(_) => true,
                None => false,
            })
        })
        .map(|(y, _)| y + 1)
        .unwrap_or(0)
}

fn get_tallest_rock_from_cave_at_index(cave: &Cave, index: usize) -> usize {
    cave.iter()
        .enumerate()
        .rev()
        .find(|(_, row)| {
            row.get(index)
                .filter(|space| match space {
                    Some(_) => true,
                    None => false,
                })
                .is_some()
        })
        .map(|(y, _)| y + 1)
        .unwrap_or(0)
}

fn add_height_padding(mut cave: Cave) -> Cave {
    let tallest_rock = get_tallest_rock_from_cave(&cave);

    while cave.len() - tallest_rock < 3 {
        cave.push_back(vec![None, None, None, None, None, None, None])
    }

    while cave.len() - tallest_rock > 3 {
        cave.pop_back();
    }

    cave
}

fn print_cave(cave: &Cave) {
    for y in (0..cave.len()).rev() {
        let row = cave.get(y).expect("Row exists");
        println!(
            "{}",
            row.iter()
                .map(|value| match value {
                    Some(RockType::Active) => "@",
                    Some(RockType::Solid) => "#",
                    None => ".",
                })
                .join("")
        )
    }
}
