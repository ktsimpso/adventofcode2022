use adventofcode2022::{
    parse_between_blank_lines, parse_lines, parse_usize, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    text, Parser,
};
use clap::ArgMatches;
use itertools::Itertools;
use std::{
    cell::LazyCell,
    cmp::{max, min},
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    iter::once,
};

type ParseOutput = Vec<Valve>;

pub const DAY_16: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day16",
        "Finds the maximum amount of pressure you can release in the given time peroid.",
        "Path to the input file. Each line describes a cave. A cave has a name, pressure rate, and the caves it's connected to.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments {}, "Finds the maximum amount of pressure that can be released in 30 minutes.");
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Valve {
    name: ValveName,
    rate: u16,
    connections: Vec<ValveName>,
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_valve()).then_ignore(end())
}

fn parse_valve() -> impl Parser<char, Valve, Error = Simple<char>> {
    just("Valve ")
        .ignore_then(parse_valve_name())
        .then_ignore(just(" has flow rate="))
        .then(parse_usize())
        .then_ignore(just("; tunnels lead to valves ").or(just("; tunnel leads to valve ")))
        .then(parse_valve_name().separated_by(just(", ")))
        .map(|((name, rate), connections)| Valve {
            name,
            rate: rate as u16,
            connections,
        })
}

fn parse_valve_name() -> impl Parser<char, ValveName, Error = Simple<char>> {
    text::ident().map(|name| ValveName::new(name))
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let valves = input
        .into_iter()
        .map(|valve| (valve.name, valve))
        .collect::<HashMap<_, _>>();
    let graph = Graph {
        nodes: valves.clone(),
    };
    let start = ValveName::new("AA".to_string());
    let target_nodes = graph
        .nodes
        .values()
        .filter(|valve| valve.rate > 0)
        .map(|valve| valve.name.clone())
        .collect::<Vec<_>>();
    let paths = target_nodes
        .clone()
        .into_iter()
        .chain(once(start))
        .map(|source| {
            (
                source,
                target_nodes
                    .iter()
                    .filter_map(|target| {
                        shortest_path(vec![source], &graph, target)
                            .map(|path| (*target, path.nodes.len() as u16))
                    })
                    .collect::<HashMap<_, _>>(),
            )
        })
        .collect::<HashMap<_, _>>();

    best_pressure_possible(
        start,
        30,
        BTreeSet::from([start]),
        &valves,
        &paths,
        &mut HashMap::new(),
    ) as usize
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct ValveName(u16);

impl ValveName {
    fn new(name: String) -> ValveName {
        let mut item = 0;
        name.chars()
            .into_iter()
            .enumerate()
            .for_each(|(index, char)| item |= (char as u16) << (8 * index as u16));
        ValveName(item)
    }
}

fn best_pressure_possible(
    current: ValveName,
    minutes_left: u16,
    visited: BTreeSet<ValveName>,
    valves: &HashMap<ValveName, Valve>,
    paths: &HashMap<ValveName, HashMap<ValveName, u16>>,
    cache: &mut HashMap<(ValveName, u16, BTreeSet<ValveName>), u16>,
) -> u16 {
    match cache.get(&(current, minutes_left, visited.clone())) {
        Some(result) => *result,
        None => {
            let result = paths
                .get(&current)
                .into_iter()
                .flat_map(|current_paths| current_paths.into_iter())
                .filter(|(path, _)| !visited.contains(path))
                .filter(|(_, distance)| minutes_left >= **distance)
                .map(|(next_valve, distance)| {
                    let new_minutes_left = minutes_left - distance;
                    let mut new_visited = visited.clone();
                    new_visited.insert(*next_valve);
                    let current_rate =
                        valves.get(&next_valve).map(|v| v.rate).unwrap_or(0) * new_minutes_left;
                    current_rate
                        + best_pressure_possible(
                            *next_valve,
                            new_minutes_left,
                            new_visited,
                            valves,
                            paths,
                            cache,
                        )
                })
                .max()
                .unwrap_or(0);
            cache.insert((current, minutes_left, visited), result);
            result
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
struct ValveDistance {
    target: ValveName,
    distance: u16,
}

#[derive(Debug, Clone)]
struct Graph {
    nodes: HashMap<ValveName, Valve>,
}

#[derive(Debug, Clone)]
struct Path {
    nodes: Vec<ValveName>,
}

fn shortest_path(start_items: Vec<ValveName>, graph: &Graph, target: &ValveName) -> Option<Path> {
    let mut visited = start_items.iter().cloned().collect::<BTreeSet<ValveName>>();

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
            .get(last)
            .expect("Valid index")
            .connections
            .iter()
            .filter(|node| !visited.contains(node))
            .cloned()
            .collect::<Vec<_>>();

        filtered_adjacents
            .into_iter()
            .map(|index| (index, graph.nodes.get(&index).expect("Valid index")))
            .for_each(|(index, node)| {
                let mut new = current.clone();
                new.nodes.push(index);
                if node.name == *target {
                    match &final_path {
                        Some(current_best) => {
                            if current_best.nodes.len() > new.nodes.len() {
                                final_path = Some(new)
                            }
                        }
                        None => final_path = Some(new),
                    }
                } else {
                    visited.insert(index);
                    queue.push_back(new);
                }
            });
    }
    final_path
}
