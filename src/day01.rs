use adventofcode2022::{parse_lines, parse_usize, Problem, ProblemWithTwoParts};
use bpaf::{construct, short, Parser as ArgParser};
use chumsky::{prelude::Simple, Parser};

type ParseOutput = Vec<Option<usize>>;

pub const DAY_01: ProblemWithTwoParts<CommandLineArguments, ParseOutput, usize> = Problem::new(
    "day01",
    "Takes a list of elves backpacks calorie count and find the ones with the most",
    "Path to the input file. Input should be newline delimited groups integers. Each group represents one elf's bag, each line in the group is the caloric value of that item.",
    parse_arguments,
    parse_file,
    run,
)
.with_part1(CommandLineArguments { n: 1}, "Finds the elf with the most calories in their bag and returns the sum of the calories")
.with_part2(CommandLineArguments { n: 3 }, "Finds the elves with the 3 top most calories and sums the calories.");

#[derive(Debug, Clone)]
pub struct CommandLineArguments {
    n: usize,
}

fn parse_arguments() -> Box<dyn ArgParser<CommandLineArguments>> {
    let n = short('n')
        .long("number")
        .help("The number of elves to sum")
        .argument::<usize>("SHORT");
    Box::new(construct!(CommandLineArguments { n }))
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

    sums.sort();
    sums.reverse();

    sums.into_iter().take(arguments.n).sum()
}
