use adventofcode2022::{flag_arg, parse_isize, parse_lines, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::{end, just},
    text, Parser,
};
use clap::ArgMatches;
use std::{cell::LazyCell, collections::HashMap};

type ParseOutput = Vec<Monkey>;

pub const DAY_21: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let equal = flag_arg("equal", 'e', "The number of elves to sum");
    let problem = Problem::new(
        "day21",
        "Evaludates the monkey expression",
        "Path to the input file. Each line has a monkey name, followed by an expression.",
        vec![equal],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments { equal: false },
        "Finds the value of the expression called by root.",
    )
    .with_part2(
        CommandLineArguments { equal: true },
        "Finds the value needs to equal what root wants to use.",
    );
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    equal: bool,
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    CommandLineArguments {
        equal: *args.get_one::<bool>("equal").unwrap_or(&false),
    }
}

#[derive(Debug, Clone)]
pub struct Monkey {
    name: String,
    operation: Operation,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Value(isize),
    Experssion(String, String, Operator),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_monkey()).then_ignore(end())
}

fn parse_monkey() -> impl Parser<char, Monkey, Error = Simple<char>> {
    text::ident()
        .then_ignore(just(": "))
        .then(parse_operation())
        .map(|(name, operation)| Monkey { name, operation })
}

fn parse_operation() -> impl Parser<char, Operation, Error = Simple<char>> {
    let value = parse_isize().map(|value| Operation::Value(value));
    let expression = text::ident()
        .then_ignore(just(" "))
        .then(parse_operator())
        .then_ignore(just(" "))
        .then(text::ident())
        .map(|((first, operator), second)| Operation::Experssion(first, second, operator));
    value.or(expression)
}

fn parse_operator() -> impl Parser<char, Operator, Error = Simple<char>> {
    let add = just("+").to(Operator::Add);
    let sub = just("-").to(Operator::Sub);
    let mul = just("*").to(Operator::Mul);
    let div = just("/").to(Operator::Div);
    add.or(sub).or(mul).or(div)
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> isize {
    let mut monkeys = input
        .into_iter()
        .map(|monkey| (monkey.name.clone(), monkey))
        .collect::<HashMap<_, _>>();

    if arguments.equal {
        let _humn = monkeys.remove("humn").expect("human is me");
        let root = monkeys.remove("root").expect("I am root");
        match root.operation {
            Operation::Experssion(left, right, _) => {
                match (
                    evaluate_monkey_opt(&left, &monkeys),
                    evaluate_monkey_opt(&right, &monkeys),
                ) {
                    (None, Some(target)) => resolve_to_target(&left, target, &monkeys),
                    (Some(target), None) => resolve_to_target(&right, target, &monkeys),
                    (None, None) => 0,
                    (Some(_), Some(_)) => 0,
                }
            }
            Operation::Value(value) => value,
        }
    } else {
        evaluate_monkey(&"root".to_string(), &monkeys)
    }
}

fn evaluate_monkey(name: &String, monkeys: &HashMap<String, Monkey>) -> isize {
    let current = monkeys.get(name).expect("Monkey exists");

    match &current.operation {
        Operation::Value(value) => *value,
        Operation::Experssion(sub1, sub2, operator) => match operator {
            Operator::Add => evaluate_monkey(&sub1, monkeys) + evaluate_monkey(&sub2, monkeys),
            Operator::Sub => evaluate_monkey(&sub1, monkeys) - evaluate_monkey(&sub2, monkeys),
            Operator::Mul => evaluate_monkey(&sub1, monkeys) * evaluate_monkey(&sub2, monkeys),
            Operator::Div => evaluate_monkey(&sub1, monkeys) / evaluate_monkey(&sub2, monkeys),
        },
    }
}

fn evaluate_monkey_opt(name: &String, monkeys: &HashMap<String, Monkey>) -> Option<isize> {
    monkeys
        .get(name)
        .and_then(|current| match &current.operation {
            Operation::Value(value) => Some(*value),
            Operation::Experssion(sub1, sub2, operator) => match (
                evaluate_monkey_opt(&sub1, monkeys),
                evaluate_monkey_opt(&sub2, monkeys),
            ) {
                (Some(left), Some(right)) => Some(match operator {
                    Operator::Add => left + right,
                    Operator::Sub => left - right,
                    Operator::Mul => left * right,
                    Operator::Div => left / right,
                }),
                _ => None,
            },
        })
}

fn resolve_to_target(name: &String, target: isize, monkeys: &HashMap<String, Monkey>) -> isize {
    if name == "humn" {
        return target;
    }
    let current = monkeys.get(name).expect("Monkey exists");

    match &current.operation {
        Operation::Value(_) => 0,
        Operation::Experssion(left, right, operation) => match (
            evaluate_monkey_opt(&left, monkeys),
            evaluate_monkey_opt(&right, monkeys),
        ) {
            (None, None) => 0,
            (None, Some(value)) => {
                let new_target = match operation {
                    Operator::Add => target - value,
                    Operator::Sub => target + value,
                    Operator::Mul => target / value,
                    Operator::Div => target * value,
                };
                resolve_to_target(&left, new_target, monkeys)
            }
            (Some(value), None) => {
                let new_target = match operation {
                    Operator::Add => target - value,
                    Operator::Sub => value - target,
                    Operator::Mul => target / value,
                    Operator::Div => value / target,
                };
                resolve_to_target(&right, new_target, monkeys)
            }
            (Some(_), Some(_)) => 0,
        },
    }
}
