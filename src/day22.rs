use adventofcode2022::{
    parse_lines, parse_usize, single_arg, BoundedPoint, Command, ParseError, PointDirection,
    Problem, RotationDegrees,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    text, Parser,
};
use clap::{value_parser, ArgMatches};
use std::{
    cell::LazyCell,
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
};

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
            let region_path_graph = build_region_path_graph(&region_bounds, region_size);
            let region_paths_3d = build_3d_region_paths(&region_path_graph);
            traverse_grid_cube(
                &board,
                &input.1,
                &region_bounds,
                max_x - 1,
                max_y - 1,
                region_size,
                &get_region_rotation_mappings(&region_paths_3d),
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

fn build_region_path_graph(
    region_bounds: &HashMap<usize, (usize, usize)>,
    region_size: usize,
) -> HashMap<usize, Vec<(usize, PointDirection)>> {
    region_bounds
        .iter()
        .map(|(region, (x, y))| {
            (
                *region,
                region_bounds
                    .iter()
                    .filter(move |(next_region, _)| region != *next_region)
                    .filter_map(|(next_region, (next_x, next_y))| {
                        if x + region_size == *next_x && y == next_y {
                            Some((*next_region, PointDirection::Right))
                        } else if x > &0 && x - region_size == *next_x && y == next_y {
                            Some((*next_region, PointDirection::Left))
                        } else if x == next_x && y + region_size == *next_y {
                            Some((*next_region, PointDirection::Down))
                        } else if y > &0 && x == next_x && y - region_size == *next_y {
                            Some((*next_region, PointDirection::Up))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        })
        .collect()
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
                    ..*point
                },
                PointDirection::Down => BoundedPoint {
                    x: x_min + x_offset,
                    y: *y_min,
                    ..*point
                },
                PointDirection::Left => BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + y_offset,
                    ..*point
                },
                PointDirection::Right => BoundedPoint {
                    x: *x_min,
                    y: y_min + y_offset,
                    ..*point
                },
            },
            direction.clone(),
        ),
        RotationDegrees::Ninety => match direction {
            PointDirection::Up => (
                BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + region_offset - x_offset,
                    ..*point
                },
                PointDirection::Left,
            ),
            PointDirection::Down => (
                BoundedPoint {
                    x: *x_min,
                    y: y_min + region_offset - x_offset,
                    ..*point
                },
                PointDirection::Right,
            ),
            PointDirection::Left => (
                BoundedPoint {
                    x: x_min + y_offset,
                    y: *y_min,
                    ..*point
                },
                PointDirection::Down,
            ),
            PointDirection::Right => (
                BoundedPoint {
                    x: x_min + y_offset,
                    y: y_min + region_offset,
                    ..*point
                },
                PointDirection::Up,
            ),
        },
        RotationDegrees::OneHundredEighty => match direction {
            PointDirection::Up => (
                BoundedPoint {
                    x: x_min + region_offset - x_offset,
                    y: *y_min,
                    ..*point
                },
                PointDirection::Down,
            ),
            PointDirection::Down => (
                BoundedPoint {
                    x: x_min + region_offset - x_offset,
                    y: y_min + region_offset,
                    ..*point
                },
                PointDirection::Up,
            ),
            PointDirection::Left => (
                BoundedPoint {
                    x: *x_min,
                    y: y_min + region_offset - y_offset,
                    ..*point
                },
                PointDirection::Right,
            ),
            PointDirection::Right => (
                BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + region_offset - y_offset,
                    ..*point
                },
                PointDirection::Left,
            ),
        },
        RotationDegrees::TwoHundredSeventy => match direction {
            PointDirection::Up => (
                BoundedPoint {
                    x: *x_min,
                    y: y_min + x_offset,
                    ..*point
                },
                PointDirection::Right,
            ),
            PointDirection::Down => (
                BoundedPoint {
                    x: x_min + region_offset,
                    y: y_min + x_offset,
                    ..*point
                },
                PointDirection::Left,
            ),
            PointDirection::Left => (
                BoundedPoint {
                    x: x_min + region_offset - y_offset,
                    y: y_min + region_offset,
                    ..*point
                },
                PointDirection::Up,
            ),
            PointDirection::Right => (
                BoundedPoint {
                    x: x_min + region_offset - y_offset,
                    y: *y_min,
                    ..*point
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
        RotationDegrees::Zero => (direction.get_opposite(), RotationDegrees::Zero),
        RotationDegrees::Ninety => (
            direction.get_clockwise(),
            RotationDegrees::TwoHundredSeventy,
        ),
        RotationDegrees::OneHundredEighty => (direction, rotation),
        RotationDegrees::TwoHundredSeventy => {
            (direction.get_counter_clockwise(), RotationDegrees::Ninety)
        }
    };

    to.insert(new_direction, (from_region, new_rotation));
}

fn get_mapping_from_path(path: &Vec<PointDirection>) -> Option<(PointDirection, RotationDegrees)> {
    match path.as_slice() {
        [p1] => Some((*p1, RotationDegrees::Zero)),
        [p1, p2] if p1 == p2 => None,
        [p1, p2] => Some((*p2, p1.get_rotation(p2))),
        [p1, p2, p3] if p1 == p2 && p2 == p3 => Some((p1.get_opposite(), RotationDegrees::Zero)),
        [p1, _, p3] if p1 == p3 => None,
        [p1, p2, p3] if p2 == p3 => Some((p1.get_opposite(), RotationDegrees::OneHundredEighty)),
        [p1, p2, p3] if p1 == p2 => Some((*p3, RotationDegrees::OneHundredEighty)),
        [p1, p2, p3, p4] if p1 == p4 && p2 == p3 => None,
        [p1, p2, p3, p4] if p1 == p2 && p2 == p3 => Some((*p4, p4.get_rotation(p1))),
        [p1, p2, p3, p4] if p2 == p3 && p3 == p4 => Some((p4.get_opposite(), p4.get_rotation(p1))),
        [p1, p2, p3, p4] if p1 == p2 && p2 == p4 => Some((p1.get_opposite(), p3.get_rotation(p1))),
        [p1, p2, p3, p4] if p1 == p3 && p3 == p4 => Some((p2.get_opposite(), p1.get_rotation(p2))),
        [p1, p2, p3, p4] if p1 == p3 && p2 == p4 => Some((p1.get_opposite(), p2.get_rotation(p1))),
        [p1, p2, p3, p4, p5] if p1 == p5 && p2 == p3 && p3 == p4 => None,
        [p1, p2, p3, p4, p5] if p1 == p2 && p2 == p4 && p4 == p5 => {
            Some((p3.get_opposite(), RotationDegrees::Zero))
        }
        [p1, p2, p3, p4, p5] if p1 == p4 && p2 == p3 && p3 == p5 => {
            Some((p2.get_opposite(), RotationDegrees::Zero))
        }
        [p1, p2, p3, p4, p5] if p1 == p3 && p3 == p4 && p2 == p5 => {
            Some((p1.get_opposite(), RotationDegrees::Zero))
        }
        [p1, p2, p3, p4, p5] if p1 == p3 && p3 == p5 && p2 == p4 => {
            Some((p2.get_opposite(), RotationDegrees::Zero))
        }
        _ => None,
    }
}

fn get_region_rotation_mappings(
    paths: &HashMap<usize, Vec<(usize, Vec<PointDirection>)>>,
) -> HashMap<usize, HashMap<PointDirection, (usize, RotationDegrees)>> {
    let mut mappings = HashMap::new();

    let first = paths.get(&1).expect("First face exists");
    let mut opposite = 1;
    let mut visitied = HashSet::from([1]);

    first.iter().for_each(|(region, path)| {
        if let Some((direction, rotation)) = get_mapping_from_path(path) {
            insert_mapping(&mut mappings, 1, *region, direction, rotation);
        } else {
            opposite = *region;
        }
    });

    visitied.insert(opposite);

    let next = paths.get(&opposite).expect("Opposite side exists");
    next.iter()
        .filter(|(region, _)| !visitied.contains(region))
        .for_each(|(region, path)| {
            if let Some((direction, rotation)) = get_mapping_from_path(path) {
                insert_mapping(&mut mappings, opposite, *region, direction, rotation);
            }
        });

    let next_region = paths
        .keys()
        .find(|region| !visitied.contains(region))
        .expect("At least one unvisited region");
    visitied.insert(*next_region);

    let next = paths.get(&next_region).expect("Opposite side exists");

    next.iter()
        .filter(|(region, _)| !visitied.contains(region))
        .for_each(|(region, path)| {
            if let Some((direction, rotation)) = get_mapping_from_path(path) {
                insert_mapping(&mut mappings, *next_region, *region, direction, rotation);
            } else {
                opposite = *region
            }
        });

    visitied.insert(opposite);

    let next = paths.get(&opposite).expect("Opposite side exists");
    next.iter()
        .filter(|(region, _)| !visitied.contains(region))
        .for_each(|(region, path)| {
            if let Some((direction, rotation)) = get_mapping_from_path(path) {
                insert_mapping(&mut mappings, opposite, *region, direction, rotation);
            }
        });

    mappings
}

fn build_3d_region_paths(
    paths: &HashMap<usize, Vec<(usize, PointDirection)>>,
) -> HashMap<usize, Vec<(usize, Vec<PointDirection>)>> {
    paths
        .keys()
        .map(|region| {
            (
                *region,
                paths
                    .keys()
                    .filter(|target_region| region != *target_region)
                    .map(|target_region| {
                        (
                            *target_region,
                            shortest_path(*region, paths, target_region)
                                .nodes
                                .into_iter()
                                .map(|(_, direction)| direction)
                                .collect(),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

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
            Instruction::RotateClockwise => current_direction = current_direction.get_clockwise(),
            Instruction::RotateCounterClockwise => {
                current_direction = current_direction.get_counter_clockwise()
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
            Instruction::RotateClockwise => current_direction = current_direction.get_clockwise(),
            Instruction::RotateCounterClockwise => {
                current_direction = current_direction.get_counter_clockwise()
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

#[derive(Debug, Clone)]
struct Path {
    nodes: Vec<(usize, PointDirection)>,
}

fn shortest_path(
    start_region: usize,
    graph: &HashMap<usize, Vec<(usize, PointDirection)>>,
    target: &usize,
) -> Path {
    let mut visited = BTreeSet::from([start_region]);

    let mut queue = graph
        .get(&start_region)
        .iter()
        .flat_map(|adjacents| adjacents.iter().cloned())
        .map(|step| Path { nodes: vec![step] })
        .collect::<VecDeque<Path>>();

    while queue.len() > 0 {
        let current = queue.pop_front().expect("At least one item in the queue");
        let last = current.nodes.last().expect("At least one item in the path");

        if &last.0 == target {
            return current;
        }

        let filtered_adjacents = graph
            .get(&last.0)
            .expect("Valid index")
            .iter()
            .filter(|node| !visited.contains(&node.0))
            .cloned()
            .collect::<Vec<_>>();

        filtered_adjacents
            .into_iter()
            .for_each(|(region, direction)| {
                let mut new = current.clone();
                new.nodes.push((region, direction));
                visited.insert(region);
                queue.push_back(new);
            });
    }
    unreachable!()
}
