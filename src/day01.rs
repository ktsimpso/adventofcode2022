use adventofcode2022::{parse_lines, parse_usize, Problem, ProblemWithOnePart};
use bpaf::{construct, short, Parser as ArgParser};
use chumsky::{prelude::Simple, Parser};

type ParseOutput = Vec<Option<usize>>;

pub const DAY_01: ProblemWithOnePart<CommandLineArguments, ParseOutput, usize> = Problem::new(
    "day01",
    "Takes a list of elves backpacks calorie count and find the ones with the most",
    "Path to the input file. Input should be newline delimited groups integers. Each group represents one elf's bag, each line in the group is the caloric value of that item.",
    parse_arguments,
    parse_file,
    run,
)
.with_part1(CommandLineArguments {}, "Finds the elf with the most calories in their bag and returns the sum of the calories");
//.with_part2(Arguments1 { s: 3 }, "Docs for part2");

#[derive(Debug, Clone)]
pub struct CommandLineArguments {}

fn parse_arguments() -> Box<dyn ArgParser<CommandLineArguments>> {
    //let s = short('s').help("Test argument").argument::<usize>("SHORT");
    Box::new(construct!(CommandLineArguments {}))
}

fn parse_file(file: String) -> ParseOutput {
    parser().parse(file).unwrap()
}

fn parser() -> impl Parser<char, ParseOutput, Error = Simple<char>> {
    parse_lines(parse_usize().or_not())
}

fn run(input: ParseOutput, arguments: CommandLineArguments) -> usize {
    let (mut sums, last_sum) = input.into_iter().fold(
        (Vec::new(), 0usize),
        |(mut results, mut current_sum), current| {
            match current {
                Some(value) => current_sum += value,
                _ => {
                    results.push(current_sum);
                    current_sum = 0;
                }
            };
            (results, current_sum)
        },
    );

    sums.push(last_sum);

    sums.into_iter().max().unwrap_or(0usize)
}
