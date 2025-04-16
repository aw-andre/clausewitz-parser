mod cli;
mod database;
mod parser;

use clap::Parser;
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // read from CLI and validate
    let args = cli::Cli::parse();
    args.validate();

    // load database URL, connect, and initialize
    let pool = Arc::new(PgPool::connect(&args.database_url).await?);
    database::initialize(&pool).await?;

    // parse files and add data to database
    let mut handles = Vec::new();
    for file in args.files {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            database::insert_file(file, &pool);
        }));
    }

    // finish all jobs
    for job in handles {
        job.await.unwrap();
    }
    Ok(())
}
