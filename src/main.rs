mod cli;
mod database;
mod processor;

use clap::Parser;
use pest::iterators::Pair;
use processor::{ParsedFile, Rule, UnparsedFile};
use sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // read from CLI and validate
    let args = cli::Cli::parse();
    args.validate();

    // load database URL and connect
    let pool = PgPool::connect(&args.database_url).await?;

    // parse files and add data to database
    for file in &args.files {
        let unparsedfile = UnparsedFile::new(file);
        let parsedfile = unparsedfile.process();

        database::insert_file(parsedfile, &pool);
    }
    Ok(())
}
