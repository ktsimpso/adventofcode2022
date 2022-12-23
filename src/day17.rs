use adventofcode2022::{single_arg, BoundedPoint, Command, ParseError, PointDirection, Problem};
use anyhow::Result;
use chumsky::{prelude::Simple, primitive::end, primitive::just, text, Parser};
use clap::ArgMatches;
use itertools::Itertools;
use std::{
    cell::LazyCell,
    collections::{HashSet, VecDeque},
};

type ParseOutput = Vec<PointDirection>;

pub const DAY_17: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let number = single_arg("number", 'n', "The number of rocks that fall")
        .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day17",
        "Finds the height of falling rocks after a number of rocks have fallen",
        "Path to the input file. The wind direction at any given iteration. Cycles to the start once input ends.",
        vec![number],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { n: 2022 }, "Finds the height of the rock tower after 2022 iterations.")
    .with_part2(CommandLineArguments { n: 1_000_000_000_000}, "Finds the height of the rock tower after 1_000_000_000_000 iterations.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    n: usize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        n: *args.get_one::<usize>("number").expect("Valid arguments"),
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
    let wind_length = input.len();

    let rocks = [horizontal, plus, chair, vertical, square]
        .into_iter()
        .cycle();
    let wind = input.into_iter().cycle();

    let cave: Cave = VecDeque::from(vec![vec![None, None, None, None, None, None, None]]);

    drop_rocks(arguments.n, cave, rocks, wind, wind_length)
}

fn drop_rocks(
    n: usize,
    mut cave: Cave,
    mut rocks: impl Iterator<Item = Rock>,
    mut wind: impl Iterator<Item = PointDirection>,
    wind_loop: usize,
) -> usize {
    let mut count = 0;
    let mut wind_count = 0;
    let mut prev_count = 0;
    let mut cycle_count = 0;
    let mut prev_i = 0;
    let mut consecutive_count_equal = 0isize;
    let mut current_cycle_count = 0usize;
    let mut target_cycle_size = 1usize;

    let mut wind_indexes = HashSet::<usize>::new();
    let mut prev_wind_indexes = HashSet::<usize>::new();
    let mut wind_index_match_count = 0isize;
    let mut current_wind_cycle_count = 0usize;
    let mut target_wind_cycle_size = 1usize;
    let mut valid_indexes_found = false;
    let mut wind_index = 0;

    let mut i = 0;

    while i < n {
        let next_rock = rocks.next().expect("Rock exists");
        cave = add_rock(cave, next_rock.clone());
        let mut rock_complete = false;

        while !rock_complete {
            let wind_direction = wind.next().expect("window blows");
            wind_count += 1;
            if can_move_rock(&cave, &wind_direction) {
                cave = move_rock(cave, wind_direction)
            }

            if can_move_rock(&cave, &PointDirection::Up) {
                cave = move_rock(cave, PointDirection::Up);
            } else {
                cave = freeze_rock(cave);
                if !valid_indexes_found {
                    wind_indexes.insert(wind_count % wind_loop);
                } else if valid_indexes_found && wind_count % wind_loop == wind_index {
                    current_wind_cycle_count += 1;
                    if current_wind_cycle_count == target_wind_cycle_size {
                        current_cycle_count += 1;

                        if current_cycle_count == target_cycle_size {
                            if cycle_count == count - prev_count {
                                consecutive_count_equal += 1;
                            } else {
                                consecutive_count_equal -= 1;
                            }

                            if consecutive_count_equal == -3 {
                                target_cycle_size += 1;
                                consecutive_count_equal = 0;
                            } else if consecutive_count_equal == 3 {
                                let cycle_length = i - prev_i;
                                println!("Cycle detected after {} iterations. Cycle has a size of {} and a length of {}", i, cycle_count, cycle_length);
                                let cycle_values = (n - i) / cycle_length;
                                count += cycle_values * cycle_count;
                                i += cycle_values * cycle_length;
                            }

                            current_cycle_count = 0;
                            cycle_count = count - prev_count;
                            prev_count = count;
                            prev_i = i;
                        }

                        current_wind_cycle_count = 0;
                    }
                }
                rock_complete = true;
            };

            if !valid_indexes_found && wind_count % wind_loop == 0 {
                current_wind_cycle_count += 1;

                if current_wind_cycle_count == target_wind_cycle_size {
                    if wind_indexes == prev_wind_indexes {
                        wind_index_match_count += 1;
                    } else {
                        wind_index_match_count -= 1;
                    }

                    prev_wind_indexes = wind_indexes;
                    wind_indexes = HashSet::new();

                    if wind_index_match_count == 3 {
                        wind_index = *prev_wind_indexes
                            .iter()
                            .next()
                            .expect("At least one matching");
                        valid_indexes_found = true;
                        println!(
                            "Found stable wind drop cycles every {} wind cycles. Using: {} to find rock cycles",
                            target_wind_cycle_size, wind_index
                        );
                        prev_count = 0;
                        prev_i = 0;
                    } else if wind_index_match_count == -3 {
                        target_wind_cycle_size += 1;
                        wind_index_match_count = 0;
                    }

                    current_wind_cycle_count = 0;
                }
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
                cave = cave.split_off(split_point - 1);
            }
        }

        i += 1;
    }

    count + get_tallest_rock_from_cave(&cave)
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

fn _print_cave(cave: &Cave) {
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
