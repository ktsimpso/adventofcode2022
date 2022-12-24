use adventofcode2022::{
    parse_lines, single_arg, BoundedPoint, Command, ParseError, PointDirection, Problem,
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
    collections::{BTreeMap, HashSet, VecDeque},
    iter::once,
};

type ParseOutput = Vec<Vec<Tile>>;

pub const DAY_24: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let traversals = single_arg(
        "traversals",
        't',
        "The number of times to traverse the snow storm",
    )
    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day24",
        "Figures out how long it will take the elves to navigatea snow storm.",
        "Path to the input file. The current status of the field with the direction blizzards are traveling.",
        vec![traversals],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { traversals: 1 }, "Finds how long it takes to traverse the snow storm")
    .with_part2(CommandLineArguments { traversals: 3 }, "Finds how long it takes to traverse the snow storm 3 times.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    traversals: usize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        traversals: *args
            .get_one::<usize>("traversals")
            .expect("Valid arguments"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    Wall,
    Blizzard(PointDirection),
    Empty,
    Expedition,
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
    let wall = just("#").to(Tile::Wall);
    let empty = just(".").to(Tile::Empty);
    let blizzard = parse_direction().map(|direction| Tile::Blizzard(direction));
    let expedition = just("E").to(Tile::Expedition);

    wall.or(empty).or(blizzard).or(expedition)
}

fn parse_direction() -> impl Parser<char, PointDirection, Error = Simple<char>> {
    let left = just("<").to(PointDirection::Left);
    let right = just(">").to(PointDirection::Right);
    let down = just("v").to(PointDirection::Down);
    let up = just("^").to(PointDirection::Up);

    left.or(right).or(down).or(up)
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let max_x = input.iter().map(|row| row.len() - 1).max().unwrap_or(0);
    let max_y = input.len() - 1;

    let mut start_point = input
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, tile)| (BoundedPoint { x, y, max_x, max_y }, tile))
        })
        .find(|(_, tile)| match tile {
            Tile::Empty => true,
            _ => false,
        })
        .map(|(point, _)| point)
        .expect("At least one blank");

    let mut target_point = input
        .iter()
        .enumerate()
        .last()
        .and_then(|(y, row)| {
            row.iter()
                .enumerate()
                .find(|(_, tile)| match tile {
                    Tile::Empty => true,
                    _ => false,
                })
                .map(|(x, _)| BoundedPoint { x, y, max_x, max_y })
        })
        .expect("Exit point exists");

    let map = input
        .into_iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.into_iter()
                .enumerate()
                .filter(|(_, tile)| match tile {
                    Tile::Empty => false,
                    _ => true,
                })
                .map(move |(x, tile)| (BoundedPoint { x, y, max_x, max_y }, tile))
        })
        .fold(
            BTreeMap::<BoundedPoint, Vec<Tile>>::new(),
            |mut acc, (point, tile)| {
                let entry = acc.entry(point).or_insert(Vec::new());
                entry.push(tile);
                acc
            },
        );

    let mut previous = None;
    let start = Some(map.clone());
    let mut next = run_movements(map.clone());
    let mut all_maps = VecDeque::new();

    while previous != start {
        all_maps.push_back(next.clone());
        previous = Some(next.clone());
        next = run_movements(next);
    }

    all_maps.rotate_left(all_maps.len() - 1);

    println!("Found {} possible storm patterns", all_maps.len());

    let mut count = 0;
    for _ in 0..arguments.traversals {
        count = find_path(start_point, target_point, count, &all_maps);
        (start_point, target_point) = (target_point, start_point);
    }

    count - 1
}

fn find_path(
    start_point: BoundedPoint,
    target_point: BoundedPoint,
    count: usize,
    all_maps: &VecDeque<BTreeMap<BoundedPoint, Vec<Tile>>>,
) -> usize {
    let mut queue = VecDeque::from([(start_point, count)]);
    let mut cache = HashSet::new();

    while queue.len() > 0 {
        let (current_expedition, movement) = queue.pop_front().expect("Queue is not empty");
        if current_expedition == target_point {
            return movement;
        }

        valid_expedition_movements(&current_expedition, movement, all_maps)
            .into_iter()
            .for_each(|next| {
                if !cache.contains(&(next.0, next.1 % all_maps.len())) {
                    queue.push_back(next.clone());
                    cache.insert((next.0, next.1 % all_maps.len()));
                }
            });
    }

    unreachable!()
}

fn valid_expedition_movements(
    expedition: &BoundedPoint,
    movement: usize,
    all_maps: &VecDeque<BTreeMap<BoundedPoint, Vec<Tile>>>,
) -> Vec<(BoundedPoint, usize)> {
    let next_movement = all_maps
        .get(movement % all_maps.len())
        .expect("Index exists");

    expedition
        .into_iter_cardinal_adjacent()
        .chain(once(expedition.clone()))
        .filter(|point| {
            next_movement.get(point).iter().all(|tiles| {
                tiles.iter().all(|tile| match tile {
                    Tile::Wall => false,
                    Tile::Blizzard(_) => false,
                    Tile::Empty => true,
                    Tile::Expedition => true,
                })
            })
        })
        .map(|new_expedition| (new_expedition, movement + 1))
        .collect()
}

fn run_movements(map: BTreeMap<BoundedPoint, Vec<Tile>>) -> BTreeMap<BoundedPoint, Vec<Tile>> {
    map.iter()
        .flat_map(|(point, tiles)| {
            tiles.iter().map(|tile| match tile {
                Tile::Blizzard(direction) => {
                    let mut next_point = point.get_adjacent_wrapping(direction);
                    while map.get(&next_point).iter().any(|items| {
                        items.iter().any(|tile| match tile {
                            Tile::Wall => true,
                            _ => false,
                        })
                    }) {
                        next_point = next_point.get_adjacent_wrapping(direction);
                    }
                    (next_point, tile)
                }
                _ => (point.clone(), tile),
            })
        })
        .fold(
            BTreeMap::<BoundedPoint, Vec<Tile>>::new(),
            |mut acc, (point, tile)| {
                let entry = acc.entry(point).or_insert(Vec::new());
                entry.push(tile.clone());
                acc
            },
        )
}

fn _print_map(map: &BTreeMap<BoundedPoint, Vec<Tile>>, max_x: usize, max_y: usize) {
    for y in 0..=max_y {
        for x in 0..=max_x {
            let fallback = vec![Tile::Empty];
            let point = map
                .get(&BoundedPoint { x, y, max_x, max_y })
                .unwrap_or(&fallback);
            match &point[..] {
                [tile] => match tile {
                    Tile::Wall => print!("#"),
                    Tile::Blizzard(direction) => match direction {
                        PointDirection::Up => print!("^"),
                        PointDirection::Down => print!("v"),
                        PointDirection::Left => print!("<"),
                        PointDirection::Right => print!(">"),
                    },
                    Tile::Empty => print!("."),
                    Tile::Expedition => print!("E"),
                },
                _ => print!("{}", point.len()),
            }
        }
        println!()
    }
}
