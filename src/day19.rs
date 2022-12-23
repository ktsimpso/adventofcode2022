use adventofcode2022::{parse_lines, parse_usize, single_arg, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    Parser,
};
use clap::{ArgMatches, ValueEnum};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::{
    cell::LazyCell,
    cmp::{max, min},
    collections::HashMap,
};

type ParseOutput = Vec<Blueprint>;

pub const DAY_19: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let time = single_arg("time", 't', "The time you have to crack geodes")
        .value_parser(clap::value_parser!(u16));
    let limit = single_arg("limit", 'l', "Limits the number of blueprints to check")
        .required(false)
        .value_parser(clap::value_parser!(usize));
    let stats = single_arg(
        "stats",
        's',
        "What metric shoudl be determined for the blueprints",
    )
    .value_parser(clap::value_parser!(BlueprintStats));
    let problem = Problem::new(
        "day19",
        "Finds the maximum amount of geodes you can crack with a given recipe.",
        "Path to the input file. Each line should contain a recpiee for how to constuct robots of all types.",
        vec![time, limit, stats],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { time: 24, limit: None, blueprint_stats: BlueprintStats::QualityLevelSum }, "Determines the quality level of all blueprints for 24 minutes then sums them.")
    .with_part2(CommandLineArguments { time: 32, limit: Some(3), blueprint_stats: BlueprintStats::ProductGeodes }, "Takes the first 3 blueprints and multiplioes the number of geodes found in 32 minutes");
    Box::new(problem)
});

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum BlueprintStats {
    QualityLevelSum,
    ProductGeodes,
}

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    time: u16,
    limit: Option<usize>,
    blueprint_stats: BlueprintStats,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        time: *args.get_one::<u16>("time").expect("Valid arguments"),
        limit: args.get_one::<usize>("limit").copied(),
        blueprint_stats: args
            .get_one::<BlueprintStats>("stats")
            .expect("Valid arguments")
            .clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Material {
    Ore,
    Clay,
    Obsidian,
}

#[derive(Debug, Clone)]
pub struct OreRobot {
    cost: Cost,
}

#[derive(Debug, Clone)]
pub struct ClayRobot {
    cost: Cost,
}

#[derive(Debug, Clone)]
pub struct ObsidianRobot {
    cost1: Cost,
    cost2: Cost,
}

#[derive(Debug, Clone)]
pub struct GeodeRobot {
    cost1: Cost,
    cost2: Cost,
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    id: u16,
    ore: OreRobot,
    clay: ClayRobot,
    obsidian: ObsidianRobot,
    geode: GeodeRobot,
}

#[derive(Debug, Clone)]
struct Cost {
    material: Material,
    cost: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Resources {
    ore: u16,
    clay: u16,
    obsidian: u16,
    ore_robots: u16,
    clay_robots: u16,
    obsidian_robots: u16,
    geode_robots: u16,
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_blueprint()).then_ignore(end())
}

fn parse_blueprint() -> impl Parser<char, Blueprint, Error = Simple<char>> {
    just("Blueprint ")
        .ignore_then(parse_usize())
        .then_ignore(just(": "))
        .then(parse_ore_robot())
        .then(parse_clay_robot())
        .then(parse_obsidian_robot())
        .then(parse_geode_robot())
        .map(|((((id, ore), clay), obsidian), geode)| Blueprint {
            id: id as u16,
            ore,
            clay,
            obsidian,
            geode,
        })
}

fn parse_geode_robot() -> impl Parser<char, GeodeRobot, Error = Simple<char>> {
    just("Each geode robot costs ")
        .ignore_then(parse_cost())
        .then_ignore(just(" and "))
        .then(parse_cost())
        .then_ignore(just("."))
        .map(|(cost1, cost2)| GeodeRobot { cost1, cost2 })
}

fn parse_obsidian_robot() -> impl Parser<char, ObsidianRobot, Error = Simple<char>> {
    just("Each obsidian robot costs ")
        .ignore_then(parse_cost())
        .then_ignore(just(" and "))
        .then(parse_cost())
        .then_ignore(just(". "))
        .map(|(cost1, cost2)| ObsidianRobot { cost1, cost2 })
}

fn parse_clay_robot() -> impl Parser<char, ClayRobot, Error = Simple<char>> {
    just("Each clay robot costs ")
        .ignore_then(parse_cost())
        .then_ignore(just(". "))
        .map(|cost| ClayRobot { cost })
}

fn parse_ore_robot() -> impl Parser<char, OreRobot, Error = Simple<char>> {
    just("Each ore robot costs ")
        .ignore_then(parse_cost())
        .then_ignore(just(". "))
        .map(|cost| OreRobot { cost })
}

fn parse_cost() -> impl Parser<char, Cost, Error = Simple<char>> {
    parse_usize()
        .then_ignore(just(" "))
        .then(parse_material())
        .map(|(cost, material)| Cost {
            material,
            cost: cost as u16,
        })
}

fn parse_material() -> impl Parser<char, Material, Error = Simple<char>> {
    let ore = just("ore").to(Material::Ore);
    let clay = just("clay").to(Material::Clay);
    let obsidian = just("obsidian").to(Material::Obsidian);

    ore.or(clay).or(obsidian)
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let mut count = 0;
    let result = input
        .into_iter()
        .take_while(|_| {
            let result = arguments.limit.map(|limit| count < limit).unwrap_or(true);
            count += 1;
            result
        })
        .par_bridge()
        .map(|blueprint| score_blueprint(&blueprint, arguments.time, &arguments.blueprint_stats));

    match arguments.blueprint_stats {
        BlueprintStats::QualityLevelSum => result.sum::<u16>() as usize,
        BlueprintStats::ProductGeodes => result.product::<u16>() as usize,
    }
}

fn score_blueprint(blueprint: &Blueprint, time: u16, blueprint_stats: &BlueprintStats) -> u16 {
    let start_resources = Resources {
        ore: 0,
        clay: 0,
        obsidian: 0,
        ore_robots: 1,
        clay_robots: 0,
        obsidian_robots: 0,
        geode_robots: 0,
    };

    let result = if blueprint_stats == &BlueprintStats::QualityLevelSum {
        blueprint.id
    } else {
        1
    } * get_number_of_geodes_cracked(
        blueprint,
        time,
        start_resources,
        &mut HashMap::new(),
        &get_max_material(blueprint, &Material::Ore),
        &get_max_material(blueprint, &Material::Clay),
        &get_max_material(blueprint, &Material::Obsidian),
    );
    result
}

fn get_number_of_geodes_cracked(
    blueprint: &Blueprint,
    time: u16,
    resources: Resources,
    cache: &mut HashMap<(u16, Resources), u16>,
    max_ore: &u16,
    max_clay: &u16,
    max_obsidion: &u16,
) -> u16 {
    if time == 0 {
        return 0;
    }

    match cache.get(&(time, resources.clone())) {
        Some(value) => *value,
        None => {
            let result = resources.geode_robots
                + get_purchases(blueprint, &resources, &max_ore, &max_clay, &max_obsidion)
                    .into_iter()
                    .map(|new_resources| {
                        get_number_of_geodes_cracked(
                            blueprint,
                            time - 1,
                            new_resources,
                            cache,
                            max_ore,
                            max_clay,
                            max_obsidion,
                        )
                    })
                    .max()
                    .unwrap_or(0);
            cache.insert((time, resources), result);
            result
        }
    }
}

fn get_max_material(blueprint: &Blueprint, material: &Material) -> u16 {
    let mut maximum = match &blueprint.ore.cost.material {
        m if m == material => blueprint.ore.cost.cost,
        _ => 0,
    };
    maximum = max(
        maximum,
        match &blueprint.clay.cost.material {
            m if m == material => blueprint.clay.cost.cost,
            _ => 0,
        },
    );
    maximum = max(
        maximum,
        match &blueprint.obsidian.cost1.material {
            m if m == material => blueprint.obsidian.cost1.cost,
            _ => 0,
        },
    );
    maximum = max(
        maximum,
        match &blueprint.obsidian.cost2.material {
            m if m == material => blueprint.obsidian.cost2.cost,
            _ => 0,
        },
    );
    maximum = max(
        maximum,
        match &blueprint.geode.cost1.material {
            m if m == material => blueprint.geode.cost1.cost,
            _ => 0,
        },
    );
    max(
        maximum,
        match &blueprint.geode.cost2.material {
            m if m == material => blueprint.geode.cost2.cost,
            _ => 0,
        },
    )
}

fn get_purchases(
    blueprint: &Blueprint,
    resources: &Resources,
    max_ore: &u16,
    max_clay: &u16,
    max_obsidion: &u16,
) -> Vec<Resources> {
    let make_ore = if get_purchasable_ore_robots(blueprint, resources) > 0
        && &resources.ore_robots < max_ore
    {
        let mut new_ore_resources = resources.clone();
        match blueprint.ore.cost.material {
            Material::Ore => new_ore_resources.ore -= blueprint.ore.cost.cost,
            Material::Clay => new_ore_resources.clay -= blueprint.ore.cost.cost,
            Material::Obsidian => new_ore_resources.obsidian -= blueprint.ore.cost.cost,
        }
        new_ore_resources.ore_robots += 1;
        Some(new_ore_resources)
    } else {
        None
    };

    let make_clay = if get_purchasable_clay_robots(blueprint, resources) > 0
        && &resources.clay_robots < max_clay
    {
        let mut new_clay_resources = resources.clone();
        match blueprint.clay.cost.material {
            Material::Ore => new_clay_resources.ore -= blueprint.clay.cost.cost,
            Material::Clay => new_clay_resources.clay -= blueprint.clay.cost.cost,
            Material::Obsidian => new_clay_resources.obsidian -= blueprint.clay.cost.cost,
        };

        new_clay_resources.clay_robots += 1;
        Some(new_clay_resources)
    } else {
        None
    };

    let make_obsidian = if get_purchasable_obsidian_robots(blueprint, resources) > 0
        && &resources.obsidian_robots < max_obsidion
    {
        let mut new_obsidian_resources = resources.clone();
        match blueprint.obsidian.cost1.material {
            Material::Ore => new_obsidian_resources.ore -= blueprint.obsidian.cost1.cost,
            Material::Clay => new_obsidian_resources.clay -= blueprint.obsidian.cost1.cost,
            Material::Obsidian => new_obsidian_resources.obsidian -= blueprint.obsidian.cost1.cost,
        };
        match blueprint.obsidian.cost2.material {
            Material::Ore => new_obsidian_resources.ore -= blueprint.obsidian.cost2.cost,
            Material::Clay => new_obsidian_resources.clay -= blueprint.obsidian.cost2.cost,
            Material::Obsidian => new_obsidian_resources.obsidian -= blueprint.obsidian.cost2.cost,
        };

        new_obsidian_resources.obsidian_robots += 1;
        Some(new_obsidian_resources)
    } else {
        None
    };

    let make_geode = if get_purchasable_geode_robots(blueprint, resources) > 0 {
        let mut new_geode_resources = resources.clone();
        match blueprint.geode.cost1.material {
            Material::Ore => new_geode_resources.ore -= blueprint.geode.cost1.cost,
            Material::Clay => new_geode_resources.clay -= blueprint.geode.cost1.cost,
            Material::Obsidian => new_geode_resources.obsidian -= blueprint.geode.cost1.cost,
        };
        match blueprint.geode.cost2.material {
            Material::Ore => new_geode_resources.ore -= blueprint.geode.cost2.cost,
            Material::Clay => new_geode_resources.clay -= blueprint.geode.cost2.cost,
            Material::Obsidian => new_geode_resources.obsidian -= blueprint.geode.cost2.cost,
        };

        new_geode_resources.geode_robots += 1;
        Some(new_geode_resources)
    } else {
        None
    };

    let base_resources = if get_purchasable_ore_robots(blueprint, resources) > 0
        && get_purchasable_clay_robots(blueprint, resources) > 0
        && get_purchasable_obsidian_robots(blueprint, resources) > 0
        && get_purchasable_geode_robots(blueprint, resources) > 0
    {
        None
    } else {
        Some(resources.clone())
    };

    make_ore
        .into_iter()
        .chain(make_clay.into_iter())
        .chain(make_obsidian.into_iter())
        .chain(make_geode.into_iter())
        .chain(base_resources.into_iter())
        .map(|mut new_resource| {
            new_resource.ore += resources.ore_robots;
            new_resource.clay += resources.clay_robots;
            new_resource.obsidian += resources.obsidian_robots;
            new_resource
        })
        .collect::<Vec<_>>()
}

fn get_purchasable_ore_robots(blueprint: &Blueprint, resources: &Resources) -> u16 {
    match blueprint.ore.cost.material {
        Material::Ore => resources.ore / blueprint.ore.cost.cost,
        Material::Clay => resources.clay / blueprint.ore.cost.cost,
        Material::Obsidian => resources.obsidian / blueprint.ore.cost.cost,
    }
}

fn get_purchasable_clay_robots(blueprint: &Blueprint, resources: &Resources) -> u16 {
    match blueprint.clay.cost.material {
        Material::Ore => resources.ore / blueprint.clay.cost.cost,
        Material::Clay => resources.clay / blueprint.clay.cost.cost,
        Material::Obsidian => resources.obsidian / blueprint.clay.cost.cost,
    }
}

fn get_purchasable_obsidian_robots(blueprint: &Blueprint, resources: &Resources) -> u16 {
    min(
        match blueprint.obsidian.cost1.material {
            Material::Ore => resources.ore / blueprint.obsidian.cost1.cost,
            Material::Clay => resources.clay / blueprint.obsidian.cost1.cost,
            Material::Obsidian => resources.obsidian / blueprint.obsidian.cost1.cost,
        },
        match blueprint.obsidian.cost2.material {
            Material::Ore => resources.ore / blueprint.obsidian.cost2.cost,
            Material::Clay => resources.clay / blueprint.obsidian.cost2.cost,
            Material::Obsidian => resources.obsidian / blueprint.obsidian.cost2.cost,
        },
    )
}

fn get_purchasable_geode_robots(blueprint: &Blueprint, resources: &Resources) -> u16 {
    min(
        match blueprint.geode.cost1.material {
            Material::Ore => resources.ore / blueprint.geode.cost1.cost,
            Material::Clay => resources.clay / blueprint.geode.cost1.cost,
            Material::Obsidian => resources.obsidian / blueprint.geode.cost1.cost,
        },
        match blueprint.geode.cost2.material {
            Material::Ore => resources.ore / blueprint.geode.cost2.cost,
            Material::Clay => resources.clay / blueprint.geode.cost2.cost,
            Material::Obsidian => resources.obsidian / blueprint.geode.cost2.cost,
        },
    )
}
