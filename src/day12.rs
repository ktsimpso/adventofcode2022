use adventofcode2022::{flag_arg, parse_lines, BoundedPoint, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just, one_of},
    Parser,
};
use clap::ArgMatches;
use std::{
    cell::LazyCell,
    collections::{BTreeSet, VecDeque},
};

type ParseOutput = Vec<Vec<MountainTile>>;

pub const DAY_12: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let expand = flag_arg(
        "expand",
        'e',
        "Expands the possible start positions to include 'a'",
    );
    let problem = Problem::new(
        "day12",
        "Finds the shortest path to the end goal on the mountain.",
        "Path to the input file. File should consist of all lower case letters and one S and E.",
        vec![expand],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments { expand: false },
        "Finds the shortest path between S and E.",
    )
    .with_part2(
        CommandLineArguments { expand: true },
        "Finds the shortest path between any S, or 'a' to E.",
    );
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    expand: bool,
}

#[derive(Debug, Clone)]
pub enum MountainTile {
    Base(char),
    Start,
    End,
}

impl MountainTile {
    fn get_value(&self) -> usize {
        match self {
            MountainTile::Base(value) => *value as usize - 'a' as usize,
            MountainTile::Start => 0,
            MountainTile::End => 'z' as usize - 'a' as usize,
        }
    }
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        expand: *args.get_one::<bool>("expand").expect("Valid arguments"),
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_mountain_tile().repeated().at_least(1)).then_ignore(end())
}

fn parse_mountain_tile() -> impl Parser<char, MountainTile, Error = Simple<char>> {
    let alphabet = ('a'..='z').collect::<String>();
    let base = one_of(alphabet).map(|c| MountainTile::Base(c));
    let start = just('S').to(MountainTile::Start);
    let end = just('E').to(MountainTile::End);

    base.or(start).or(end)
}

#[derive(Debug, Clone)]
struct Graph {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
struct Node {
    value: MountainTile,
    position: BoundedPoint,
    adjacents: Vec<usize>,
}

#[derive(Debug, Clone)]
struct Path {
    nodes: Vec<usize>,
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let graph = build_graph(input);

    let start = graph
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| match node.value {
            MountainTile::Start => true,
            MountainTile::Base(_) => arguments.expand && node.value.get_value() == 0,
            _ => false,
        })
        .map(|(index, _)| index)
        .collect();

    shortest_path(start, &graph)
        .expect("At least one path")
        .nodes
        .len()
        - 1
}

fn shortest_path(start_items: Vec<usize>, graph: &Graph) -> Option<Path> {
    let mut visited = start_items.iter().cloned().collect::<BTreeSet<usize>>();

    let mut queue = start_items
        .into_iter()
        .map(|node| Path { nodes: vec![node] })
        .collect::<VecDeque<Path>>();

    let mut final_path: Option<Path> = None;

    while queue.len() > 0 {
        let current = queue.pop_front().expect("At least one item in the queue");

        match &final_path {
            Some(current_best) => {
                if current_best.nodes.len() < current.nodes.len() {
                    continue;
                }
            }
            None => (),
        }

        let last = current.nodes.last().expect("At least one item in the path");
        let filtered_adjacents = graph
            .nodes
            .get(*last)
            .expect("Valid index")
            .adjacents
            .iter()
            .filter(|node| !visited.contains(node))
            .cloned()
            .collect::<Vec<_>>();

        filtered_adjacents
            .into_iter()
            .map(|index| (index, graph.nodes.get(index).expect("Valid index")))
            .for_each(|(index, node)| {
                let mut new = current.clone();
                new.nodes.push(index);
                match node.value {
                    MountainTile::End => match &final_path {
                        Some(current_best) => {
                            if current_best.nodes.len() > new.nodes.len() {
                                final_path = Some(new)
                            }
                        }
                        None => final_path = Some(new),
                    },
                    _ => {
                        visited.insert(index);
                        queue.push_back(new);
                    }
                }
            });
    }
    final_path
}

fn build_graph(mountain: Vec<Vec<MountainTile>>) -> Graph {
    let max_x = mountain.get(0).expect("At least 1 row").len() - 1;
    let max_y = mountain.len() - 1;
    let mut graph = Graph {
        nodes: mountain
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().map(move |(x, value)| Node {
                    value: value.clone(),
                    position: BoundedPoint {
                        x: x,
                        y: y,
                        max_x: max_x,
                        max_y: max_y,
                    },
                    adjacents: Vec::new(),
                })
            })
            .collect(),
    };
    graph
        .nodes
        .iter_mut()
        .for_each(|node| find_adjacent_nodes(node, &mountain));
    graph
}

fn find_adjacent_nodes(node: &mut Node, mountain: &Vec<Vec<MountainTile>>) {
    node.position
        .into_iter_cardinal_adjacent()
        .filter_map(|other| {
            mountain
                .get(other.y)
                .and_then(|row| row.get(other.x))
                .map(|tile| (other, tile))
        })
        .for_each(|(other, tile)| {
            if tile.get_value() <= (node.value.get_value() + 1) {
                node.adjacents.push(point_to_node(&other))
            }
        });
}

fn point_to_node(point: &BoundedPoint) -> usize {
    point.y * (point.max_x + 1) + point.x
}
