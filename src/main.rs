mod cli;
mod database;
mod processor;

use clap::Parser;
use pest::iterators::Pair;
use processor::{ParsedFile, Rule, UnparsedFile};
use sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // load database URL and connect
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("Error: DATABASE_URL is not set");
    let pool = PgPool::connect(&database_url).await?;

    // read filenames from CLI and validate
    let args = cli::Cli::parse();
    args.validate();

    // parse files and add data to database
    for file in &args.files {
        let unparsedfile = UnparsedFile::new(file);
        let parsedfile = unparsedfile.process();

        database::insert_file(parsedfile, &pool);
    }
    Ok(())
}
