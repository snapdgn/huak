use huak::errors::CliResult;

mod cli;
mod commands;
mod pyproject;
mod utils;

#[cfg(test)]
mod test_utils;

fn main() -> CliResult {
    cli::main()
}
