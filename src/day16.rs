use adventofcode2022::{parse_lines, parse_usize, single_arg, Command, ParseError, Problem};
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
    cmp::min,
    collections::{BTreeSet, HashMap, VecDeque},
    iter::once,
};

type ParseOutput = Vec<Valve>;

pub const DAY_16: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let time = single_arg("time", 't', "The time to release the pressure.")
        .value_parser(clap::value_parser!(u16));
    let entities = single_arg(
        "entities",
        'e',
        "The number of entities who can open valves.",
    )
    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day16",
        "Finds the maximum amount of pressure you can release in the given time peroid.",
        "Path to the input file. Each line describes a cave. A cave has a name, pressure rate, and the caves it's connected to.",
        vec![time, entities],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { time: 30, entities: 1 }, "Finds the maximum amount of pressure that can be released in 30 minutes by 1 enitity.")
    .with_part2(CommandLineArguments { time: 26, entities: 2 }, "Finds the maximum amount of pressure that can be released in 26 minutes by 2 entities.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    time: u16,
    entities: usize,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        time: *args.get_one::<u16>("time").expect("Valid arguments"),
        entities: *args.get_one::<usize>("entities").expect("Valid arguments"),
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
        vec![
            ValveDistance {
                target: start,
                distance: 0,
            };
            arguments.entities
        ],
        arguments.time,
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

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
struct ValveDistance {
    target: ValveName,
    distance: u16,
}

fn best_pressure_possible(
    destinations: Vec<ValveDistance>,
    minutes_left: u16,
    visited: BTreeSet<ValveName>,
    valves: &HashMap<ValveName, Valve>,
    paths: &HashMap<ValveName, HashMap<ValveName, u16>>,
    cache: &mut HashMap<(Vec<ValveDistance>, u16, BTreeSet<ValveName>), u16>,
) -> u16 {
    match cache.get(&(destinations.clone(), minutes_left, visited.clone())) {
        Some(result) => *result,
        None => {
            let (arrived, enroute): (Vec<_>, Vec<_>) = destinations
                .clone()
                .into_iter()
                .partition(|destination| destination.distance == 0);
            let (new_paths, _no_routes): (Vec<_>, Vec<_>) = arrived
                .into_iter()
                .map(|destination| {
                    paths
                        .get(&destination.target)
                        .into_iter()
                        .flat_map(|current_paths| current_paths.into_iter())
                        .filter(|(path, _)| !visited.contains(path))
                        .filter(|(_, distance)| minutes_left >= **distance)
                        .collect::<Vec<_>>()
                })
                .partition(|routes| routes.len() > 0);
            let combined_paths = new_paths
                .into_iter()
                .multi_cartesian_product()
                .map(|routes| {
                    routes
                        .into_iter()
                        .fold(HashMap::new(), |mut acc, route| {
                            let current_low = acc.entry(route.0).or_insert(route.1);
                            *current_low = min(&current_low, route.1);
                            acc
                        })
                        .into_iter()
                        .map(|(name, distance)| ValveDistance {
                            target: *name,
                            distance: *distance,
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let result = if combined_paths.len() > 0 {
                combined_paths
                    .into_iter()
                    .map(|destinations| {
                        let smallest_delta = destinations
                            .iter()
                            .chain(enroute.iter())
                            .map(|valve_distance| valve_distance.distance)
                            .min()
                            .expect("at least one");
                        let mut new_visited = visited.clone();
                        destinations.iter().for_each(|valve_distance| {
                            new_visited.insert(valve_distance.target);
                        });
                        let current_rate: u16 = destinations
                            .iter()
                            .map(|valve_distance| {
                                valves
                                    .get(&valve_distance.target)
                                    .map(|v| v.rate * (minutes_left - valve_distance.distance))
                                    .unwrap_or(0)
                            })
                            .sum();
                        let new_minutes_left = minutes_left - smallest_delta;
                        let moved_destinations = destinations
                            .into_iter()
                            .chain(enroute.clone().into_iter())
                            .map(|destination| ValveDistance {
                                target: destination.target,
                                distance: destination.distance - smallest_delta,
                            })
                            .collect();
                        current_rate
                            + best_pressure_possible(
                                moved_destinations,
                                new_minutes_left,
                                new_visited,
                                valves,
                                paths,
                                cache,
                            )
                    })
                    .max()
                    .unwrap_or(0)
            } else if enroute.len() > 0 {
                let smallest_delta = enroute
                    .iter()
                    .map(|valve_distance| valve_distance.distance)
                    .min()
                    .expect("at least one");
                let new_minutes_left = minutes_left - smallest_delta;
                let moved_destinations = enroute
                    .into_iter()
                    .map(|destination| ValveDistance {
                        target: destination.target,
                        distance: destination.distance - smallest_delta,
                    })
                    .collect();
                best_pressure_possible(
                    moved_destinations,
                    new_minutes_left,
                    visited.clone(),
                    valves,
                    paths,
                    cache,
                )
            } else {
                0
            };
            cache.insert((destinations, minutes_left, visited), result);

            result
        }
    }
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
