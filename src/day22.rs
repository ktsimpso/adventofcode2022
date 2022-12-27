use adventofcode2022::{
    parse_lines, parse_usize, single_arg, BoundedPoint, Command, ParseError, PointDirection,
    Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    text, Parser,
};
use clap::{value_parser, ArgMatches};
use std::{cell::LazyCell, collections::HashMap};

type ParseOutput = (Vec<Vec<Tile>>, Vec<Instruction>);

pub const DAY_22: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let region_size = single_arg(
        "region",
        'r',
        "The size of the cube faces if folded up into a cube",
    )
    .required(false)
    .value_parser(value_parser!(usize));

    let problem = Problem::new(
        "day22",
        "Traverses the path in a grid and find the final position and facing.",
        "Path to the input file. The grid where a . is an empty space, and a # is a wall. Should be a valid cube net.",
        vec![region_size],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { cubed_region_size: None }, "Finds the final position on the flat grid.")
    .with_part2(CommandLineArguments { cubed_region_size: Some(50) }, "Finds the final position on the cubed grid.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    cubed_region_size: Option<usize>,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        cubed_region_size: args.get_one::<usize>("region").cloned(),
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

#[derive(Debug, Clone)]
enum RotationDegrees {
    Zero,
    Ninety,
    OneHundredEighty,
    TwoHundredSeventy,
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

    let (point, direction) = match arguments.cubed_region_size {
        Some(region_size) => {
            let region_bounds = parse_regions_from_board(&board, region_size);
            traverse_grid_cube(
                &board,
                &input.1,
                &region_bounds,
                max_x - 1,
                max_y - 1,
                region_size,
                &get_region_rotation_mappings(),
            )
        }
        None => traverse_grid(&board, &input.1, max_x - 1, max_y - 1),
    };

    (point.y + 1) * 1000
        + (point.x + 1) * 4
        + match direction {
            PointDirection::Up => 3,
            PointDirection::Down => 1,
            PointDirection::Left => 2,
            PointDirection::Right => 0,
        }
}

fn is_on_region_boundry(
    point: &BoundedPoint,
    region_size: usize,
    direction: &PointDirection,
) -> bool {
    match direction {
        PointDirection::Up => point.y % region_size == 0,
        PointDirection::Down => point.y % region_size == region_size - 1,
        PointDirection::Left => point.x % region_size == 0,
        PointDirection::Right => point.x % region_size == region_size - 1,
    }
}

fn parse_regions_from_board(
    board: &Vec<Vec<Tile>>,
    region_size: usize,
) -> HashMap<usize, (usize, usize)> {
    let mut current_region = 1;
    let mut region_bounds = HashMap::new();
    board
        .iter()
        .enumerate()
        .filter(|(y, _)| y % region_size == 0)
        .for_each(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(x, _)| x % region_size == 0)
                .for_each(|(x, tile)| match tile {
                    Tile::Nothing => (),
                    _ => {
                        region_bounds.insert(current_region, (x, y));
                        current_region += 1;
                    }
                })
        });

    region_bounds
}

fn get_current_region(
    point: &BoundedPoint,
    region_bounds: &HashMap<usize, (usize, usize)>,
    region_size: usize,
) -> usize {
    let x = (point.x / region_size) * region_size;
    let y = (point.y / region_size) * region_size;

    *region_bounds
        .iter()
        .find(|(_, bound)| (x, y) == **bound)
        .map(|(region, _)| region)
        .expect("region exists")
}

fn region_direction_mapping(
    point: &BoundedPoint,
    region_bounds: &HashMap<usize, (usize, usize)>,
    region_size: usize,
    direction: &PointDirection,
    region_rotation_mappings: &HashMap<usize, HashMap<PointDirection, (usize, RotationDegrees)>>,
) -> (BoundedPoint, PointDirection) {
    let current_region = get_current_region(point, region_bounds, region_size);
    let region_offset = region_size - 1;
    let (current_x_min, current_y_min) = region_bounds.get(&current_region).expect("region exists");
    let x_offset = point.x - current_x_min;
    let y_offset = point.y - current_y_min;

    let (next_region, rotation) = region_rotation_mappings
        .get(&current_region)
        .and_then(|mapping| mapping.get(direction))
        .expect("Mapping exists");

    let (x_min, y_min) = region_bounds.get(next_region).expect("region exists");

    match rotation {
        RotationDegrees::Zero => (
            match direction {
                PointDirection::Up => BoundedPoint {
                    x: x_min + x_offset,
                    y: y_min + region_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Down => BoundedPoint {
                    x: x_min + x_offset,
                    y: *y_min,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Left => BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + y_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Right => BoundedPoint {
                    x: *x_min,
                    y: y_min + y_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
            },
            direction.clone(),
        ),
        RotationDegrees::Ninety => match direction {
            PointDirection::Up => (
                BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + region_offset - x_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Left,
            ),
            PointDirection::Down => (
                BoundedPoint {
                    x: *x_min,
                    y: y_min + region_offset - x_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Right,
            ),
            PointDirection::Left => (
                BoundedPoint {
                    x: x_min + y_offset,
                    y: *y_min,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Down,
            ),
            PointDirection::Right => (
                BoundedPoint {
                    x: x_min + y_offset,
                    y: y_min + region_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Up,
            ),
        },
        RotationDegrees::OneHundredEighty => match direction {
            PointDirection::Up => (
                BoundedPoint {
                    x: x_min + region_offset - x_offset,
                    y: *y_min,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Down,
            ),
            PointDirection::Down => (
                BoundedPoint {
                    x: x_min + region_offset - x_offset,
                    y: y_min + region_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Up,
            ),
            PointDirection::Left => (
                BoundedPoint {
                    x: *x_min,
                    y: y_min + region_offset - y_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Right,
            ),
            PointDirection::Right => (
                BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + region_offset - y_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Left,
            ),
        },
        RotationDegrees::TwoHundredSeventy => match direction {
            PointDirection::Up => (
                BoundedPoint {
                    x: *x_min,
                    y: y_min + x_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Right,
            ),
            PointDirection::Down => (
                BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + x_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Left,
            ),
            PointDirection::Left => (
                BoundedPoint {
                    x: x_min + region_offset - y_offset,
                    y: y_min + region_offset,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Up,
            ),
            PointDirection::Right => (
                BoundedPoint {
                    x: x_min + region_offset - y_offset,
                    y: *y_min,
                    max_x: point.max_x,
                    max_y: point.max_y,
                },
                PointDirection::Down,
            ),
        },
    }
}

fn insert_mapping(
    mapping: &mut HashMap<usize, HashMap<PointDirection, (usize, RotationDegrees)>>,
    from_region: usize,
    to_region: usize,
    direction: PointDirection,
    rotation: RotationDegrees,
) {
    let from = mapping.entry(from_region).or_insert(HashMap::new());
    from.insert(direction.clone(), (to_region, rotation.clone()));

    let to = mapping.entry(to_region).or_insert(HashMap::new());
    let (new_direction, new_rotation) = match rotation {
        RotationDegrees::Zero => (
            match direction {
                PointDirection::Up => PointDirection::Down,
                PointDirection::Down => PointDirection::Up,
                PointDirection::Left => PointDirection::Right,
                PointDirection::Right => PointDirection::Left,
            },
            RotationDegrees::Zero,
        ),
        RotationDegrees::Ninety => (
            match direction {
                PointDirection::Up => PointDirection::Right,
                PointDirection::Down => PointDirection::Left,
                PointDirection::Left => PointDirection::Up,
                PointDirection::Right => PointDirection::Down,
            },
            RotationDegrees::TwoHundredSeventy,
        ),
        RotationDegrees::OneHundredEighty => (direction, rotation),
        RotationDegrees::TwoHundredSeventy => (
            match direction {
                PointDirection::Up => PointDirection::Left,
                PointDirection::Down => PointDirection::Right,
                PointDirection::Left => PointDirection::Down,
                PointDirection::Right => PointDirection::Up,
            },
            RotationDegrees::Ninety,
        ),
    };

    to.insert(new_direction, (from_region, new_rotation));
}

// input net
fn get_region_rotation_mappings(
) -> HashMap<usize, HashMap<PointDirection, (usize, RotationDegrees)>> {
    let mut mappings = HashMap::new();

    insert_mapping(
        &mut mappings,
        1,
        3,
        PointDirection::Down,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        1,
        2,
        PointDirection::Right,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        1,
        6,
        PointDirection::Up,
        RotationDegrees::TwoHundredSeventy,
    );

    insert_mapping(
        &mut mappings,
        1,
        4,
        PointDirection::Left,
        RotationDegrees::OneHundredEighty,
    );

    insert_mapping(
        &mut mappings,
        5,
        6,
        PointDirection::Down,
        RotationDegrees::TwoHundredSeventy,
    );

    insert_mapping(
        &mut mappings,
        5,
        2,
        PointDirection::Right,
        RotationDegrees::OneHundredEighty,
    );

    insert_mapping(
        &mut mappings,
        5,
        3,
        PointDirection::Up,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        5,
        4,
        PointDirection::Left,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        4,
        6,
        PointDirection::Down,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        4,
        3,
        PointDirection::Up,
        RotationDegrees::TwoHundredSeventy,
    );

    insert_mapping(
        &mut mappings,
        2,
        3,
        PointDirection::Down,
        RotationDegrees::TwoHundredSeventy,
    );

    insert_mapping(
        &mut mappings,
        2,
        6,
        PointDirection::Up,
        RotationDegrees::Zero,
    );

    mappings
}

// sample net
/*fn get_region_rotation_mappings(
) -> HashMap<usize, HashMap<PointDirection, (usize, RotationDegrees)>> {
    let mut mappings = HashMap::new();

    insert_mapping(
        &mut mappings,
        1,
        4,
        PointDirection::Down,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        1,
        3,
        PointDirection::Left,
        RotationDegrees::Ninety,
    );

    insert_mapping(
        &mut mappings,
        1,
        6,
        PointDirection::Right,
        RotationDegrees::OneHundredEighty,
    );

    insert_mapping(
        &mut mappings,
        1,
        2,
        PointDirection::Up,
        RotationDegrees::OneHundredEighty,
    );

    insert_mapping(
        &mut mappings,
        2,
        5,
        PointDirection::Down,
        RotationDegrees::OneHundredEighty,
    );

    insert_mapping(
        &mut mappings,
        2,
        6,
        PointDirection::Left,
        RotationDegrees::TwoHundredSeventy,
    );

    insert_mapping(
        &mut mappings,
        2,
        3,
        PointDirection::Right,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        3,
        5,
        PointDirection::Down,
        RotationDegrees::Ninety,
    );

    insert_mapping(
        &mut mappings,
        3,
        4,
        PointDirection::Right,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        4,
        5,
        PointDirection::Down,
        RotationDegrees::Zero,
    );

    insert_mapping(
        &mut mappings,
        4,
        6,
        PointDirection::Right,
        RotationDegrees::TwoHundredSeventy,
    );

    insert_mapping(
        &mut mappings,
        5,
        6,
        PointDirection::Right,
        RotationDegrees::Zero,
    );

    mappings
}*/

fn traverse_grid_cube(
    board: &Vec<Vec<Tile>>,
    instructions: &Vec<Instruction>,
    region_bounds: &HashMap<usize, (usize, usize)>,
    max_x: usize,
    max_y: usize,
    region_size: usize,
    region_rotation_mappings: &HashMap<usize, HashMap<PointDirection, (usize, RotationDegrees)>>,
) -> (BoundedPoint, PointDirection) {
    let mut current_point = board
        .get(0)
        .and_then(|row| {
            row.iter().enumerate().find(|(_, tile)| match tile {
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
                    if is_on_region_boundry(&current_point, region_size, &current_direction) {
                        let (next_point, next_direction) = region_direction_mapping(
                            &current_point,
                            &region_bounds,
                            region_size,
                            &current_direction,
                            region_rotation_mappings,
                        );
                        let next_tile = board
                            .get(next_point.y)
                            .and_then(|row| row.get(next_point.x))
                            .expect("Tile Exists");
                        if next_tile == &Tile::Wall {
                            break;
                        }

                        current_point = next_point;
                        current_direction = next_direction;
                    } else {
                        let next_point = current_point.get_adjacent_wrapping(&current_direction);
                        let next_tile = board
                            .get(next_point.y)
                            .and_then(|row| row.get(next_point.x))
                            .expect("Tile Exists");

                        if next_tile == &Tile::Wall {
                            break;
                        }

                        current_point = next_point;
                    }
                }
            }
        });

    (current_point, current_direction)
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
            row.iter().enumerate().find(|(_, tile)| match tile {
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
