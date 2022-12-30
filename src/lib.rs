#![feature(once_cell)]

mod two_d_vec;
pub use two_d_vec::{BoundedPoint, PointDirection, RotationDegrees};

use anyhow::Result;
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{
    error::SimpleReason,
    prelude::Simple,
    primitive::{end, just, take_until},
    text::{self, newline},
    Parser,
};
use clap::{
    builder::PathBufValueParser, Arg, ArgAction, ArgMatches, Command as ClapCommand, ValueHint,
};
use itertools::Itertools;
use std::{
    error::Error,
    fmt::{self, Display},
    fs::File,
    io::Read,
    ops::Sub,
    path::PathBuf,
};

pub enum CommandResult {
    Isize(isize),
    Usize(usize),
    String(String),
}

impl Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandResult::Isize(val) => val.fmt(f),
            CommandResult::Usize(val) => val.fmt(f),
            CommandResult::String(val) => val.fmt(f),
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

impl From<String> for CommandResult {
    fn from(item: String) -> Self {
        CommandResult::String(item)
    }
}

#[derive(Debug)]
pub struct ParseError(pub String, pub Vec<Simple<char>>);

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", combine_parse_errors(&self.0, &self.1))
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

pub fn combine_parse_errors(source: &String, errors: &Vec<Simple<char>>) -> String {
    errors
        .iter()
        .map(|e| format_parse_error(source, &e))
        .join("\n")
}

pub fn format_parse_error(source: &String, error: &Simple<char>) -> String {
    let report = Report::build(ReportKind::Error, (), error.span().start);

    let report = match error.reason() {
        SimpleReason::Unclosed { span, delimiter } => report
            .with_message(format!(
                "Unclosed delimiter {}",
                delimiter.fg(Color::Yellow)
            ))
            .with_label(
                Label::new(span.clone())
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_color(Color::Yellow),
            )
            .with_label(
                Label::new(error.span())
                    .with_message(format!(
                        "Must be closed before this {}",
                        error
                            .found()
                            .map(|c| c.to_string())
                            .unwrap_or("end of file".escape_default().to_string())
                            .fg(Color::Red)
                    ))
                    .with_color(Color::Red),
            ),
        SimpleReason::Unexpected => report
            .with_message(format!(
                "{}, expected {}",
                if error.found().is_some() {
                    "Unexpected token in input"
                } else {
                    "Unexpected end of input"
                },
                if error.expected().len() == 0 {
                    "end of input".to_string()
                } else {
                    error
                        .expected()
                        .map(|expected| match expected {
                            Some(expected) => {
                                format!("'{}'", expected.escape_default())
                            }
                            None => "end of input".to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ))
            .with_label(
                Label::new(error.span())
                    .with_message(format!(
                        "Unexpected token '{}'",
                        error
                            .found()
                            .map(|c| c.to_string())
                            .unwrap_or("end of file".to_string())
                            .fg(Color::Red)
                    ))
                    .with_color(Color::Red),
            ),
        SimpleReason::Custom(msg) => return msg.clone(),
    };
    let mut buf = vec![];
    let finished_report = report.finish();
    finished_report
        .write(Source::from(&source), &mut buf)
        .unwrap();
    std::str::from_utf8(&buf[..]).unwrap().to_string()
}

pub trait Command {
    fn run(&self, args: &ArgMatches) -> Result<CommandResult>;

    fn get_name(&self) -> &'static str;

    fn get_subcommand(&self) -> ClapCommand;
}

pub struct Problem<T, U, R>
where
    T: Clone,
    R: Into<CommandResult>,
{
    name: &'static str,
    subcommand: ClapCommand,
    part1_data: Option<T>,
    part2_data: Option<T>,
    parse_args: fn(&ArgMatches) -> T,
    parse_file: fn(String) -> Result<U>,
    run: fn(U, T) -> R,
}

impl<T, U, R> Problem<T, U, R>
where
    T: Clone,
    R: Into<CommandResult>,
{
    pub fn new(
        name: &'static str,
        help: &str,
        file_help: &str,
        args: Vec<Arg>,
        parse_args: fn(&ArgMatches) -> T,
        parse_file: fn(String) -> Result<U>,
        run: fn(U, T) -> R,
    ) -> Self {
        let subcommand = subcommand(name, help, file_help).args(args);
        Problem {
            name,
            subcommand,
            part1_data: None,
            part2_data: None,
            parse_args,
            parse_file,
            run,
        }
    }

    pub fn with_part1(mut self, argument: T, docs: &str) -> Self {
        self.subcommand = self.subcommand.with_part1(docs);
        self.part1_data = Some(argument);
        self
    }

    pub fn with_part2(mut self, argument: T, docs: &str) -> Self {
        self.subcommand = self.subcommand.with_part2(docs);
        self.part2_data = Some(argument);
        self
    }

    fn parse_matches(&self, args: &ArgMatches) -> (String, T) {
        let (file_path, arg) = match args.subcommand_name() {
            Some(name) => {
                let mut file = PathBuf::new();
                file.push(format!("{}/input.txt", self.name));
                let arg = self
                    .part1_data
                    .iter()
                    .map(|arg| ("part1", arg))
                    .chain(self.part2_data.iter().map(|arg| ("part2", arg)))
                    .filter_map(|(part_name, arg)| match name {
                        a if a == part_name => Some(arg),
                        _ => None,
                    })
                    .next()
                    .expect("At least one part");

                (file, arg.clone())
            }
            _ => (
                args.get_one::<PathBuf>("file")
                    .expect("File is required")
                    .clone(),
                (self.parse_args)(args),
            ),
        };
        let file_contents = file_to_string(&file_path).expect("Can read file");
        (file_contents, arg)
    }
}

impl<T, U, R> Command for Problem<T, U, R>
where
    T: Clone,
    R: Into<CommandResult>,
{
    fn run(&self, args: &ArgMatches) -> Result<CommandResult> {
        let (file_contents, arg) = self.parse_matches(args);
        let parse_result = (self.parse_file)(file_contents);
        parse_result.map(|parsed| (self.run)(parsed, arg).into())
    }

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn get_subcommand(&self) -> ClapCommand {
        self.subcommand.clone()
    }
}

fn file_arg(help: &str) -> Arg {
    single_arg("file", 'f', help)
        .value_hint(ValueHint::FilePath)
        .value_parser(PathBufValueParser::new())
}

pub fn single_arg(name: &'static str, short: char, help: &str) -> Arg {
    Arg::new(name)
        .short(short)
        .long(name)
        .num_args(1)
        .help(help.to_string())
        .required(true)
        .action(ArgAction::Set)
        .value_name(name.to_ascii_uppercase())
}

pub fn flag_arg(name: &'static str, short: char, help: &str) -> Arg {
    Arg::new(name)
        .short(short)
        .long(name)
        .help(help.to_string())
        .num_args(0)
        .action(ArgAction::SetTrue)
}

fn subcommand(name: &'static str, help: &str, file_help: &str) -> ClapCommand {
    ClapCommand::new(name)
        .about(help.to_string())
        .arg_required_else_help(true)
        .subcommand_negates_reqs(true)
        .arg(file_arg(file_help))
}

trait PartSubcommands {
    fn with_part1(self, docs: &str) -> Self;

    fn with_part2(self, docs: &str) -> Self;
}

impl PartSubcommands for ClapCommand {
    fn with_part1(self, docs: &str) -> Self {
        self.subcommand(ClapCommand::new("part1").about(docs.to_string()))
    }

    fn with_part2(self, docs: &str) -> Self {
        self.subcommand(ClapCommand::new("part2").about(docs.to_string()))
    }
}

pub fn file_to_string(file_name: &PathBuf) -> Result<String, std::io::Error> {
    File::open(file_name).and_then(|mut file| {
        let mut result = String::new();
        file.read_to_string(&mut result).map(|_| result)
    })
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
    line_parser.separated_by(text::newline()).allow_trailing()
}

pub fn parse_chunks<T>(
    chunker: impl Parser<char, Vec<Vec<char>>, Error = Simple<char>>,
    chunk_parser: impl Parser<char, T, Error = Simple<char>>,
) -> impl Parser<char, Vec<T>, Error = Simple<char>> {
    chunker.try_map(move |chunks, span| {
        chunks
            .clone()
            .into_iter()
            .map(|chunk| {
                chunk_parser.parse(chunk.clone()).map_err(|errors| {
                    let input = chunk.into_iter().collect::<String>();
                    let message = errors
                        .into_iter()
                        .map(|error| format_parse_error(&input, &error))
                        .join("\n");
                    Simple::custom(span.clone(), message)
                })
            })
            .collect::<Result<Vec<T>, _>>()
    })
}

pub fn parse_between_blank_lines<T>(
    chunk_parser: impl Parser<char, T, Error = Simple<char>>,
) -> impl Parser<char, Vec<T>, Error = Simple<char>> {
    parse_chunks(chunk_blank_lines(), chunk_parser)
}

pub fn chunk_blank_lines() -> impl Parser<char, Vec<Vec<char>>, Error = Simple<char>> {
    let blank_line = newline().repeated().exactly(2).ignored();
    take_until(blank_line)
        .map(|(c, _)| c)
        .repeated()
        .then(take_until(end()))
        .map(|(mut first_chunks, (last_chunk, _))| {
            first_chunks.push(last_chunk);
            first_chunks
        })
}

pub fn absolute_difference<T>(x: T, y: T) -> T
where
    T: Sub<Output = T> + PartialOrd,
{
    if x > y {
        x - y
    } else {
        y - x
    }
}
