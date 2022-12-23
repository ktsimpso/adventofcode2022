use adventofcode2022::{
    parse_between_blank_lines, parse_isize, parse_lines, parse_usize, single_arg, Command,
    ParseError, Problem,
};
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
    //let number = single_arg("number", 'n', "The number of elves to sum")
    //    .value_parser(clap::value_parser!(usize));
    let problem = Problem::new(
        "day21",
        "Evaludates the monkey expression",
        "Path to the input file. Each line has a monkey name, followed by an expression.",
        vec![],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(
        CommandLineArguments {},
        "Finds the value of the expression called by root.",
    );
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
    let monkeys = input
        .into_iter()
        .map(|monkey| (monkey.name.clone(), monkey))
        .collect::<HashMap<_, _>>();

    evaluate_monkey(&"root".to_string(), &monkeys)
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
