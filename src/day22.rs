use adventofcode2022::{
    flag_arg, parse_lines, parse_usize, BoundedPoint, Command, ParseError, PointDirection, Problem,
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
    let cubed = flag_arg("cubed", 'c', "Should fold up the net into a cube");
    let problem = Problem::new(
        "day22",
        "Traverses the path in a grid and find the final position and facing.",
        "Path to the input file. The grid where a . is an empty space, and a # is a wall. Should be a valid cube net.",
        vec![cubed],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { cubed: false }, "Finds the final position on the flat grid.")
    .with_part2(CommandLineArguments { cubed: true }, "Finds the final position on the cubed grid.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    cubed: bool,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        cubed: *args.get_one::<bool>("cubed").unwrap_or(&false),
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

    let (point, direction) = if arguments.cubed {
        traverse_grid_cube(&board, &input.1, max_x - 1, max_y - 1, 50)
    } else {
        traverse_grid(&board, &input.1, max_x - 1, max_y - 1)
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

// input net
fn get_current_region(point: &BoundedPoint, region_size: usize) -> usize {
    if point.y < region_size {
        if point.x < region_size * 2 {
            1
        } else {
            2
        }
    } else if point.y < region_size * 2 {
        3
    } else if point.y < region_size * 3 {
        if point.x < region_size {
            4
        } else {
            5
        }
    } else {
        6
    }
}

fn get_region_bounds(region: usize) -> (usize, usize) {
    if region == 1 {
        (50, 0)
    } else if region == 2 {
        (100, 0)
    } else if region == 3 {
        (50, 50)
    } else if region == 4 {
        (0, 100)
    } else if region == 5 {
        (50, 100)
    } else {
        (0, 150)
    }
}

fn adjacent_region_mapping(
    point: &BoundedPoint,
    region_size: usize,
    direction: &PointDirection,
) -> (BoundedPoint, PointDirection) {
    let current_region = get_current_region(point, region_size);
    let region_offset = region_size - 1;
    if current_region == 1 {
        let (current_x_min, current_y_min) = get_region_bounds(1);
        let x_offset = point.x - current_x_min;
        let y_offset = point.y - current_y_min;
        match direction {
            PointDirection::Up => {
                let (x_min, y_min) = get_region_bounds(6);
                (
                    BoundedPoint {
                        x: x_min,
                        y: y_min + x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Right,
                )
            }
            PointDirection::Down => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Left => {
                let (x_min, y_min) = get_region_bounds(4);
                (
                    BoundedPoint {
                        x: x_min,
                        y: y_min + region_offset - y_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Right,
                )
            }
            PointDirection::Right => (point.get_adjacent_wrapping(direction), direction.clone()),
        }
    } else if current_region == 2 {
        let (current_x_min, current_y_min) = get_region_bounds(2);
        let x_offset = point.x - current_x_min;
        let y_offset = point.y - current_y_min;

        match direction {
            PointDirection::Up => {
                let (x_min, y_min) = get_region_bounds(6);
                (
                    BoundedPoint {
                        x: x_min + x_offset,
                        y: y_min + region_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
            PointDirection::Down => {
                let (x_min, y_min) = get_region_bounds(3);
                (
                    BoundedPoint {
                        x: x_min + region_offset,
                        y: y_min + x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Left,
                )
            }
            PointDirection::Left => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Right => {
                let (x_min, y_min) = get_region_bounds(5);
                (
                    BoundedPoint {
                        x: x_min + region_offset,
                        y: y_min + region_offset - y_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Left,
                )
            }
        }
    } else if current_region == 3 {
        let (current_x_min, current_y_min) = get_region_bounds(3);
        let _x_offset = point.x - current_x_min;
        let y_offset = point.y - current_y_min;

        match direction {
            PointDirection::Up => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Down => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Left => {
                let (x_min, y_min) = get_region_bounds(4);
                (
                    BoundedPoint {
                        x: x_min + y_offset,
                        y: y_min,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Down,
                )
            }
            PointDirection::Right => {
                let (x_min, y_min) = get_region_bounds(2);
                (
                    BoundedPoint {
                        x: x_min + y_offset,
                        y: y_min + region_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
        }
    } else if current_region == 4 {
        let (current_x_min, current_y_min) = get_region_bounds(4);
        let x_offset = point.x - current_x_min;
        let y_offset = point.y - current_y_min;

        match direction {
            PointDirection::Up => {
                let (x_min, y_min) = get_region_bounds(3);
                (
                    BoundedPoint {
                        x: x_min,
                        y: y_min + x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Right,
                )
            }
            PointDirection::Down => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Left => {
                let (x_min, y_min) = get_region_bounds(1);
                (
                    BoundedPoint {
                        x: x_min,
                        y: y_min + region_offset - y_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Right,
                )
            }
            PointDirection::Right => (point.get_adjacent_wrapping(direction), direction.clone()),
        }
    } else if current_region == 5 {
        let (current_x_min, current_y_min) = get_region_bounds(5);
        let x_offset = point.x - current_x_min;
        let y_offset = point.y - current_y_min;

        match direction {
            PointDirection::Up => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Down => {
                let (x_min, y_min) = get_region_bounds(6);
                (
                    BoundedPoint {
                        x: x_min + region_offset,
                        y: y_min + x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Left,
                )
            }
            PointDirection::Left => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Right => {
                let (x_min, y_min) = get_region_bounds(2);
                (
                    BoundedPoint {
                        x: x_min + region_offset,
                        y: y_min + region_offset - y_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Left,
                )
            }
        }
    } else {
        let (current_x_min, current_y_min) = get_region_bounds(6);
        let x_offset = point.x - current_x_min;
        let y_offset = point.y - current_y_min;

        match direction {
            PointDirection::Up => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Down => {
                let (x_min, y_min) = get_region_bounds(2);
                (
                    BoundedPoint {
                        x: x_min + x_offset,
                        y: y_min,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Down,
                )
            }
            PointDirection::Left => {
                let (x_min, y_min) = get_region_bounds(1);
                (
                    BoundedPoint {
                        x: x_min + y_offset,
                        y: y_min,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Down,
                )
            }
            PointDirection::Right => {
                let (x_min, y_min) = get_region_bounds(5);
                (
                    BoundedPoint {
                        x: x_min + y_offset,
                        y: y_min + region_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
        }
    }
}

// sample net
/*fn get_current_region(point: &BoundedPoint, region_size: usize) -> usize {
    if point.y < region_size {
        1
    } else if point.y < region_size * 2 {
        if point.x < region_size {
            2
        } else if point.x < region_size * 2 {
            3
        } else {
            4
        }
    } else {
        if point.x < region_size * 3 {
            5
        } else {
            6
        }
    }
}

fn get_region_bounds(region: usize) -> (usize, usize) {
    if region == 1 {
        (8, 0)
    } else if region == 2 {
        (0, 4)
    } else if region == 3 {
        (4, 4)
    } else if region == 4 {
        (8, 4)
    } else if region == 5 {
        (8, 8)
    } else {
        (12, 8)
    }
}

fn adjacent_region_mapping(
    point: &BoundedPoint,
    region_size: usize,
    direction: &PointDirection,
) -> (BoundedPoint, PointDirection) {
    let current_region = get_current_region(point, region_size);
    let region_offset = region_size - 1;
    if current_region == 1 {
        let (current_x_min, current_y_min) = get_region_bounds(1);
        let x_offset = point.x - current_x_min;
        let y_offet = point.y - current_y_min;
        match direction {
            PointDirection::Up => {
                let (x_min, y_min) = get_region_bounds(2);
                (
                    BoundedPoint {
                        x: x_min + region_offset - x_offset,
                        y: y_min,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Down,
                )
            }
            PointDirection::Down => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Left => {
                let (x_min, y_min) = get_region_bounds(3);
                (
                    BoundedPoint {
                        x: x_min + y_offet,
                        y: y_min,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Down,
                )
            }
            PointDirection::Right => {
                let (x_min, y_min) = get_region_bounds(6);
                (
                    BoundedPoint {
                        x: x_min + region_offset,
                        y: y_min + region_offset - y_offet,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Left,
                )
            }
        }
    } else if current_region == 2 {
        let (current_x_min, current_y_min) = get_region_bounds(2);
        let x_offset = point.x - current_x_min;
        let y_offet = point.y - current_y_min;

        match direction {
            PointDirection::Up => {
                let (x_min, y_min) = get_region_bounds(1);
                (
                    BoundedPoint {
                        x: x_min + region_offset - x_offset,
                        y: y_min,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Down,
                )
            }
            PointDirection::Down => {
                let (x_min, y_min) = get_region_bounds(5);
                (
                    BoundedPoint {
                        x: x_min + region_offset - x_offset,
                        y: y_min + region_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
            PointDirection::Left => {
                let (x_min, y_min) = get_region_bounds(6);
                (
                    BoundedPoint {
                        x: x_min + region_offset - y_offet,
                        y: y_min + region_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
            PointDirection::Right => (point.get_adjacent_wrapping(direction), direction.clone()),
        }
    } else if current_region == 3 {
        let (current_x_min, current_y_min) = get_region_bounds(3);
        let x_offset = point.x - current_x_min;
        let _y_offet = point.y - current_y_min;

        match direction {
            PointDirection::Up => {
                let (x_min, y_min) = get_region_bounds(1);
                (
                    BoundedPoint {
                        x: x_min,
                        y: y_min + x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Right,
                )
            }
            PointDirection::Down => {
                let (x_min, y_min) = get_region_bounds(5);
                (
                    BoundedPoint {
                        x: x_min,
                        y: y_min + region_offset - x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Right,
                )
            }
            PointDirection::Left => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Right => (point.get_adjacent_wrapping(direction), direction.clone()),
        }
    } else if current_region == 4 {
        let (current_x_min, current_y_min) = get_region_bounds(4);
        let _x_offset = point.x - current_x_min;
        let y_offet = point.y - current_y_min;

        match direction {
            PointDirection::Up => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Down => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Left => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Right => {
                let (x_min, y_min) = get_region_bounds(6);
                (
                    BoundedPoint {
                        x: x_min + region_offset - y_offet,
                        y: y_min,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Down,
                )
            }
        }
    } else if current_region == 5 {
        let (current_x_min, current_y_min) = get_region_bounds(5);
        let x_offset = point.x - current_x_min;
        let y_offet = point.y - current_y_min;

        match direction {
            PointDirection::Up => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Down => {
                let (x_min, y_min) = get_region_bounds(2);
                (
                    BoundedPoint {
                        x: x_min + region_offset - x_offset,
                        y: y_min + region_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
            PointDirection::Left => {
                let (x_min, y_min) = get_region_bounds(3);
                (
                    BoundedPoint {
                        x: x_min + region_offset - y_offet,
                        y: y_min + region_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
            PointDirection::Right => (point.get_adjacent_wrapping(direction), direction.clone()),
        }
    } else {
        let (current_x_min, current_y_min) = get_region_bounds(6);
        let x_offset = point.x - current_x_min;
        let y_offet = point.y - current_y_min;

        match direction {
            PointDirection::Up => {
                let (x_min, y_min) = get_region_bounds(4);
                (
                    BoundedPoint {
                        x: x_min + region_offset,
                        y: y_min + region_offset - x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Left,
                )
            }
            PointDirection::Down => {
                let (x_min, y_min) = get_region_bounds(2);
                (
                    BoundedPoint {
                        x: x_min,
                        y: y_min + region_offset - x_offset,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Right,
                )
            }
            PointDirection::Left => (point.get_adjacent_wrapping(direction), direction.clone()),
            PointDirection::Right => {
                let (x_min, y_min) = get_region_bounds(1);
                (
                    BoundedPoint {
                        x: x_min + region_offset,
                        y: y_min + region_offset - y_offet,
                        max_x: point.max_x,
                        max_y: point.max_y,
                    },
                    PointDirection::Up,
                )
            }
        }
    }
}*/

fn traverse_grid_cube(
    board: &Vec<Vec<Tile>>,
    instructions: &Vec<Instruction>,
    max_x: usize,
    max_y: usize,
    region_size: usize,
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
                        let (next_point, next_direction) = adjacent_region_mapping(
                            &current_point,
                            region_size,
                            &current_direction,
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
