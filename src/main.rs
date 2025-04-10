mod cli;
mod database;
mod processor;
use clap::Parser;
use pest::iterators::Pair;
use processor::{ParsedFile, Rule, UnparsedFile};

fn main() {
    let args = cli::Cli::parse();
    args.validate();

    for file in &args.files {
        let unparsedfile = UnparsedFile::new(file);
        let parsedfile = unparsedfile.process();
        let parsed = parsedfile.parsed;

        database::insert_file(file, parsed);
    }
}
