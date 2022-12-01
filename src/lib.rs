#![feature(return_position_impl_trait_in_trait)]

use std::{
    fmt::{self, Display},
    fs::File,
    io::Read,
    path::PathBuf,
};

use bpaf::{command, construct, parsers::ParseCommand, pure, short, Parser as ArgParser};
use chumsky::{prelude::Simple, primitive::just, text, Parser};

pub enum CommandResult {
    Isize(isize),
    Usize(usize),
}

impl Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandResult::Isize(val) => val.fmt(f),
            CommandResult::Usize(val) => val.fmt(f),
        }
    }
}

impl From<isize> for CommandResult {
    fn from(item: isize) -> Self {
        CommandResult::Isize(item)
    }
}

impl From<usize> for CommandResult {
    fn from(item: usize) -> Self {
        CommandResult::Usize(item)
    }
}

pub trait Command {
    fn run(self) -> CommandResult;
}

pub trait CommandParser {
    fn command(self) -> ParseCommand<impl Command>;
}

pub struct RunnableProblem<T, U, R>
where
    R: Into<CommandResult>,
{
    file: String,
    arguments: T,
    parse_file: fn(String) -> U,
    run: fn(U, T) -> R,
}

impl<T, U, R> Command for RunnableProblem<T, U, R>
where
    R: Into<CommandResult>,
{
    fn run(self) -> CommandResult {
        (self.run)((self.parse_file)(self.file), self.arguments).into()
    }
}

pub struct Problem<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    name: &'static str,
    about: &'static str,
    file_help: &'static str,
    argument_parser: fn() -> Box<dyn ArgParser<T>>,
    parse_file: fn(String) -> U,
    run: fn(U, T) -> R,
}

impl<T, U, R> Problem<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    pub const fn new(
        name: &'static str,
        about: &'static str,
        file_help: &'static str,
        argument_parser: fn() -> Box<dyn ArgParser<T>>,
        parse_file: fn(String) -> U,
        run: fn(U, T) -> R,
    ) -> Problem<T, U, R> {
        Problem {
            name,
            about,
            file_help,
            argument_parser,
            parse_file,
            run,
        }
    }

    pub const fn with_part1(self, value: T, docs: &'static str) -> ProblemWithOnePart<T, U, R> {
        ProblemWithOnePart {
            problem: self,
            part1: ProblemPart { value, docs },
        }
    }
}

impl<T, U, R> CommandParser for Problem<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    fn command(self) -> ParseCommand<impl Command> {
        let file_name = file_name(self.file_help);
        let parser = (self.argument_parser)();
        let arguments = construct!(file_name, parser);
        let parse_file = self.parse_file;
        let run = self.run;
        let options = construct!([arguments])
            .parse(parse_path)
            .map(move |(file, arguments)| RunnableProblem {
                file,
                arguments,
                parse_file,
                run,
            })
            .to_options();

        command(self.name, options.descr(self.about))
    }
}

pub struct ProblemWithOnePart<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    problem: Problem<T, U, R>,
    part1: ProblemPart<T>,
}

impl<T, U, R> ProblemWithOnePart<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    pub const fn with_part2(self, value: T, docs: &'static str) -> ProblemWithTwoParts<T, U, R> {
        ProblemWithTwoParts {
            part1: self,
            part2: ProblemPart { value, docs },
        }
    }
}

impl<T, U, R> CommandParser for ProblemWithOnePart<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    fn command(self) -> ParseCommand<impl Command> {
        let file_name = file_name(self.problem.file_help);
        let parser = (self.problem.argument_parser)();
        let arguments = construct!(file_name, parser);
        let part1 = part(
            "part1",
            self.part1.docs,
            self.problem.name,
            self.part1.value.clone(),
        );
        let parse_file = self.problem.parse_file;
        let run = self.problem.run;

        let options = construct!([part1, arguments])
            .parse(parse_path)
            .map(move |(file, arguments)| RunnableProblem {
                file,
                arguments,
                parse_file,
                run,
            })
            .to_options();

        command(self.problem.name, options.descr(self.problem.about))
    }
}

pub struct ProblemWithTwoParts<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    part1: ProblemWithOnePart<T, U, R>,
    part2: ProblemPart<T>,
}

impl<T, U, R> CommandParser for ProblemWithTwoParts<T, U, R>
where
    T: Clone + 'static,
    U: 'static,
    R: Into<CommandResult> + 'static,
{
    fn command(self) -> ParseCommand<impl Command> {
        let file_name = file_name(self.part1.problem.file_help);
        let parser = (self.part1.problem.argument_parser)();
        let arguments = construct!(file_name, parser);
        let part1 = part(
            "part1",
            self.part1.part1.docs,
            self.part1.problem.name,
            self.part1.part1.value.clone(),
        );
        let part2 = part(
            "part2",
            self.part2.docs,
            self.part1.problem.name,
            self.part2.value.clone(),
        );
        let parse_file = self.part1.problem.parse_file;
        let run = self.part1.problem.run;
        let options = construct!([part1, part2, arguments])
            .parse(parse_path)
            .map(move |(file, arguments)| RunnableProblem {
                file,
                arguments,
                parse_file,
                run,
            })
            .to_options();

        command(
            self.part1.problem.name,
            options.descr(self.part1.problem.about),
        )
    }
}

fn parse_path<T>((path, t): (PathBuf, T)) -> Result<(String, T), std::io::Error>
where
    T: Clone + 'static,
{
    File::open(path.as_path())
        .and_then(|mut file| {
            let mut result = String::new();
            file.read_to_string(&mut result).map(|_| result)
        })
        .map(|contents| (contents, t))
}

struct ProblemPart<T>
where
    T: Clone + 'static,
{
    value: T,
    docs: &'static str,
}

fn file_name(help: &str) -> impl ArgParser<PathBuf> {
    short('f')
        .long("file")
        .help(help)
        .argument::<PathBuf>("FILE")
    //.complete_shell
}

fn part<T>(
    name: &'static str,
    docs: &'static str,
    folder_name: &str,
    value: T,
) -> ParseCommand<(PathBuf, T)>
where
    T: Clone + 'static,
{
    let mut file = PathBuf::new();
    file.push(format!("{}/input.txt", folder_name));
    let file_parser = pure(file);
    let value_parser = pure(value);
    command(
        name,
        construct!(file_parser, value_parser)
            .to_options()
            .descr(docs),
    )
}

pub fn parse_usize() -> impl Parser<char, usize, Error = Simple<char>> {
    parse_usize_with_radix(10)
}

pub fn parse_usize_with_radix(radix: u32) -> impl Parser<char, usize, Error = Simple<char>> {
    text::int(radix).try_map(move |number: String, span| {
        usize::from_str_radix(&number, radix).map_err(|op| Simple::custom(span, op.to_string()))
    })
}

pub fn parse_isize() -> impl Parser<char, isize, Error = Simple<char>> {
    parse_isize_with_radix(10)
}

pub fn parse_isize_with_radix(radix: u32) -> impl Parser<char, isize, Error = Simple<char>> {
    just('-')
        .or_not()
        .then(text::int(radix))
        .try_map(move |(negative, number), span| {
            let combined_number = match negative {
                Some(_) => {
                    let mut c = "-".to_string();
                    c.push_str(&number);
                    c
                }
                _ => number,
            };
            isize::from_str_radix(&combined_number, radix)
                .map_err(|op| Simple::custom(span, op.to_string()))
        })
}

pub fn parse_lines<T>(
    line_parser: impl Parser<char, T, Error = Simple<char>>,
) -> impl Parser<char, Vec<T>, Error = Simple<char>> {
    line_parser.separated_by(just('\n')).allow_trailing()
}
