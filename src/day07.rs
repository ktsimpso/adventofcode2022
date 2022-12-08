use adventofcode2022::{parse_lines, parse_usize, single_arg, Command, ParseError, Problem};
use anyhow::Result;
use chumsky::{
    prelude::Simple,
    primitive::end,
    primitive::{just, take_until},
    text, Parser,
};
use clap::ArgMatches;
use std::cell::LazyCell;

type ParseOutput = Vec<TerminalOutput>;

pub const DAY_07: LazyCell<Box<dyn Command>> = LazyCell::new(|| {
    let threshold = single_arg("threshold", 't', "The largest directory to sum.")
        .value_parser(clap::value_parser!(usize));
    let space = single_arg(
        "space",
        's',
        "The space required to free up in the file system",
    )
    .value_parser(clap::value_parser!(usize))
    .conflicts_with("threshold");
    let problem = Problem::new(
        "day07",
        "Reads terminal output then gives stats on the file size for folders found in the terminal output.",
        "Path to the input file. The output of one terminal session of the elf computer.",
        vec![threshold, space],
        parse_arguments,
        parse_file,
        run,
    )
    .with_part1(CommandLineArguments { find_strategy: FindStrategy::SumThreshold { threshold: 100_000 }}, "Finds all the folder with size less than 100_000 and sums their total.")
    .with_part2(CommandLineArguments { find_strategy: FindStrategy::MinFree { space_needed: 30_000_000 } }, "Finds the smallest directory to delete to make space for 30_000_000 bytes.");
    Box::new(problem)
});

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    find_strategy: FindStrategy,
}

#[derive(Debug, Clone)]
pub enum FileCommand {
    ChangeDirectory(DirectoryDirection),
    List,
}

#[derive(Debug, Clone)]
pub enum DirectoryDirection {
    Root,
    Down(String),
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElfFile {
    Directory(String),
    File(String, usize),
}

#[derive(Debug, Clone)]
pub enum TerminalOutput {
    FileCommand(FileCommand),
    ElfFile(ElfFile),
}

#[derive(Debug, Clone)]
pub enum FindStrategy {
    SumThreshold { threshold: usize },
    MinFree { space_needed: usize },
}

fn parse_arguments(args: &ArgMatches) -> CommandLineArguments {
    let threshold =
        args.get_one::<usize>("threshold")
            .map(|threshold| FindStrategy::SumThreshold {
                threshold: *threshold,
            });
    let space = args
        .get_one::<usize>("space")
        .map(|space| FindStrategy::MinFree {
            space_needed: *space,
        });
    let find_strategy = match (threshold, space) {
        (Some(threshold), None) => threshold,
        (None, Some(space)) => space,
        _ => unreachable!(),
    };
    CommandLineArguments { find_strategy }
}

fn parse_file(file: String) -> Result<ParseOutput> {
    parser()
        .parse(file.clone())
        .map_err(|e| ParseError(file, e).into())
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_terminal_output()).then_ignore(end())
}

fn parse_terminal_output() -> impl Parser<char, TerminalOutput, Error = Simple<char>> {
    parse_file_command()
        .map(|command| TerminalOutput::FileCommand(command))
        .or(parse_elf_file().map(|file| TerminalOutput::ElfFile(file)))
}

fn parse_file_command() -> impl Parser<char, FileCommand, Error = Simple<char>> {
    just("$ ").ignore_then(
        just("ls")
            .to(FileCommand::List)
            .or(parse_directory_direction()
                .map(|direction| FileCommand::ChangeDirectory(direction))),
    )
}

fn parse_directory_direction() -> impl Parser<char, DirectoryDirection, Error = Simple<char>> {
    let root = just("/").to(DirectoryDirection::Root);
    let up = just("..").to(DirectoryDirection::Up);
    let down = take_until(text::newline().rewind())
        .map(|(name, _)| DirectoryDirection::Down(name.into_iter().collect()));
    just("cd ").ignore_then(root.or(up).or(down))
}

fn parse_elf_file() -> impl Parser<char, ElfFile, Error = Simple<char>> {
    parse_efile().or(parse_directory())
}

fn parse_efile() -> impl Parser<char, ElfFile, Error = Simple<char>> {
    parse_usize()
        .then_ignore(just(" "))
        .then(take_until(text::newline().rewind()))
        .map(|(size, (name, _))| ElfFile::File(name.into_iter().collect(), size))
}

fn parse_directory() -> impl Parser<char, ElfFile, Error = Simple<char>> {
    just("dir")
        .ignore_then(just(" "))
        .ignore_then(take_until(text::newline().rewind()))
        .map(|(name, _)| ElfFile::Directory(name.into_iter().collect()))
}

#[derive(Debug)]
struct Arena {
    files: Vec<FileSystem>,
}

#[derive(Debug, Clone)]
struct FileSystem {
    file: ElfFile,
    parent: Option<usize>,
    children: Vec<usize>,
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let root = FileSystem {
        file: ElfFile::Directory("/".to_string()),
        parent: None,
        children: Vec::new(),
    };
    let mut arena = Arena { files: vec![root] };
    let mut current = 0usize;

    input.into_iter().for_each(|output| match output {
        TerminalOutput::ElfFile(file) => {
            if arena
                .files
                .get(current)
                .expect("valid index")
                .children
                .iter()
                .find(|child| arena.files.get(**child).expect("valid index").file == file)
                .is_none()
            {
                let new_file = FileSystem {
                    file: file,
                    parent: Some(current),
                    children: Vec::new(),
                };
                arena.files.push(new_file);
                let new_index = arena.files.len() - 1;
                arena
                    .files
                    .get_mut(current)
                    .expect("valid index")
                    .children
                    .push(new_index);
            }
        }
        TerminalOutput::FileCommand(command) => match command {
            FileCommand::List => (),
            FileCommand::ChangeDirectory(direction) => match direction {
                DirectoryDirection::Root => current = 0usize,
                DirectoryDirection::Down(file_name) => {
                    let child = arena
                        .files
                        .get(current)
                        .expect("valid index")
                        .children
                        .iter()
                        .find(
                            |child| match &arena.files.get(**child).expect("valid index").file {
                                ElfFile::Directory(name) => name == &file_name,
                                _ => false,
                            },
                        )
                        .expect("valid cd");
                    current = *child
                }
                DirectoryDirection::Up => {
                    current = arena
                        .files
                        .get(current)
                        .expect("valid index")
                        .parent
                        .expect("valid parent")
                }
            },
        },
    });

    let directory_sizes = arena
        .files
        .iter()
        .filter(|file| match file.file {
            ElfFile::Directory(_) => true,
            _ => false,
        })
        .map(|dir| disk_usage(&arena, dir));

    match arguments.find_strategy {
        FindStrategy::SumThreshold { threshold } => {
            directory_sizes.filter(|value| value <= &threshold).sum()
        }
        FindStrategy::MinFree { space_needed } => {
            let max = 70_000_000_usize;
            let current = disk_usage(&arena, &arena.files.get(0).expect("Root exists"));
            let space_needed = space_needed - (max - current);

            let mut big_dirs = directory_sizes
                .filter(|value| value >= &space_needed)
                .collect::<Vec<usize>>();
            big_dirs.sort();
            *big_dirs.get(0).expect("At least one valid dir")
        }
    }
}

fn disk_usage(arena: &Arena, directory: &FileSystem) -> usize {
    directory
        .children
        .iter()
        .map(|file_index| {
            let child = arena.files.get(*file_index).expect("Valid index");

            match child.file {
                ElfFile::Directory(_) => disk_usage(arena, child),
                ElfFile::File(_, len) => len,
            }
        })
        .sum()
}
