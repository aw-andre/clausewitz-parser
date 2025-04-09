mod cli;
mod processor;
use clap::Parser;

fn main() {
    let args = cli::Cli::parse();
    args.validate();

    for file in &args.files {}
}
