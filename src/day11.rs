use adventofcode2022::{
    flag_arg, parse_between_blank_lines, parse_usize, single_arg, Command, ParseError, Problem,
};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    text, Parser,
};
use clap::ArgMatches;
use std::cell::LazyCell;

type ParseOutput = Vec<Monkey>;

pub const DAY_11: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let rounds = single_arg(
        "rounds",
        'r',
        "The number of rounds to run monkey business.",
    )
    .value_parser(clap::value_parser!(usize));
    let constant_reduction = single_arg(
        "constant",
        'c',
        "The constant number to reduce the worry level by.",
    )
    .value_parser(clap::value_parser!(usize));
    let auto_reduction =
        flag_arg("auto", 'a', "Automatically reduces the worry level.").conflicts_with("constant");
    let problem = Problem::new(
        "day11",
        "Determins the product of the two most active monkey's throwing. Worry levels may decrease either automatically or via a constant.",
        "Path to the input file. Monkey information seperated by a blank line",
        vec![rounds, constant_reduction, auto_reduction],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments { worry_level_reducation_strategy: WorryLevelReductionStrategy::Constant(3), rounds: 20 },
        "Does 20 iterations of monkey business with a constant reduction of 3.",
    )
    .with_part2(CommandLineArguments { worry_level_reducation_strategy: WorryLevelReductionStrategy::Auto, rounds: 10_000 }, "Does 10000 iterations of Monkey Business with automatic reduction.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    worry_level_reducation_strategy: WorryLevelReductionStrategy,
    rounds: usize,
}

#[derive(Debug, Clone)]
pub enum WorryLevelReductionStrategy {
    Constant(usize),
    Auto,
}

#[derive(Debug)]
pub struct Monkey {
    items: Vec<usize>,
    operation_operator: Operation,
    operation_operand: Operand,
    test_div: usize,
    test_true: usize,
    test_false: usize,
    inspect_count: usize,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add,
    Multiply,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Value(usize),
    Old,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    let constant_reduction = args
        .get_one::<usize>("constant")
        .map(|constant| WorryLevelReductionStrategy::Constant(*constant));
    let auto_reduction = args.get_one::<bool>("auto").and_then(|value| match value {
        true => Some(WorryLevelReductionStrategy::Auto),
        false => None,
    });
    let reduction_strategy = match (constant_reduction, auto_reduction) {
        (Some(constant), None) => constant,
        (None, Some(auto)) => auto,
        _ => unreachable!(),
    };
    CommandLineArguments {
        worry_level_reducation_strategy: reduction_strategy,
        rounds: *args.get_one::<usize>("rounds").expect("Valid arguments"),
    }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_between_blank_lines(parse_monkey()).then_ignore(end())
}

fn parse_monkey() -> impl Parser<char, Monkey, Error = Simple<char>> {
    just("Monkey ")
        .ignore_then(parse_usize())
        .ignore_then(just(":"))
        .ignore_then(text::newline())
        .ignore_then(parse_starting_items())
        .then(parse_operation())
        .then(parse_test())
        .map(|((starting_items, operation), test)| Monkey {
            items: starting_items,
            operation_operator: operation.0,
            operation_operand: operation.1,
            test_div: test.0,
            test_true: test.1,
            test_false: test.2,
            inspect_count: 0,
        })
}

fn parse_starting_items() -> impl Parser<char, Vec<usize>, Error = Simple<char>> {
    just("  Starting items: ")
        .ignore_then(parse_usize().separated_by(just(", ")))
        .then_ignore(text::newline())
}

fn parse_operation() -> impl Parser<char, (Operation, Operand), Error = Simple<char>> {
    let add = just("+").to(Operation::Add);
    let mul = just("*").to(Operation::Multiply);
    let value = parse_usize().map(|value| Operand::Value(value));
    let old = just("old").to(Operand::Old);
    just("  Operation: new = old ")
        .ignore_then(add.or(mul))
        .then_ignore(just(" "))
        .then(value.or(old))
        .then_ignore(text::newline())
}

fn parse_test() -> impl Parser<char, (usize, usize, usize), Error = Simple<char>> {
    let start = just("  Test: divisible by ")
        .ignore_then(parse_usize())
        .then_ignore(text::newline());
    let is_true = just("    If true: throw to monkey ")
        .ignore_then(parse_usize())
        .then_ignore(text::newline());
    let is_false = just("    If false: throw to monkey ").ignore_then(parse_usize());
    start
        .then(is_true)
        .then(is_false)
        .map(|((s, t), f)| (s, t, f))
}

fn run(mut input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let safe_mod: usize = input.iter().map(|monkey| monkey.test_div).product();
    for _ in 0..arguments.rounds {
        for index in 0..input.len() {
            let monkey = input.get_mut(index).expect("Valid index");
            monkey.inspect_count += monkey.items.len();
            let monkey_throws = monkey
                .items
                .iter()
                .map(|item| {
                    let worry_level = match (&monkey.operation_operator, &monkey.operation_operand)
                    {
                        (Operation::Add, Operand::Value(value)) => item + value,
                        (Operation::Add, Operand::Old) => item + item,
                        (Operation::Multiply, Operand::Value(value)) => item * value,
                        (Operation::Multiply, Operand::Old) => item * item,
                    };

                    let reduced_worry_level = match arguments.worry_level_reducation_strategy {
                        WorryLevelReductionStrategy::Constant(value) => worry_level / value,
                        WorryLevelReductionStrategy::Auto => worry_level % safe_mod,
                    };

                    if reduced_worry_level % &monkey.test_div == 0 {
                        (reduced_worry_level, monkey.test_true)
                    } else {
                        (reduced_worry_level, monkey.test_false)
                    }
                })
                .collect::<Vec<_>>();
            monkey.items.clear();
            monkey_throws
                .into_iter()
                .for_each(|(worry_level, target_monkey)| {
                    input
                        .get_mut(target_monkey)
                        .expect("Valid index")
                        .items
                        .push(worry_level)
                })
        }
    }

    let mut inspections = input
        .into_iter()
        .map(|monkey| monkey.inspect_count)
        .collect::<Vec<_>>();
    inspections.sort();
    inspections.reverse();
    inspections
        .into_iter()
        .take(2)
        .reduce(|acc, i| acc * i)
        .unwrap_or(0)
}
