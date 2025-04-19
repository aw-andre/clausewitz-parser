mod cli;
mod database;
mod parser;

use clap::Parser;
use sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // read from CLI and validate
    let args = cli::Cli::parse();
    args.validate();

    // load database URL, connect, and initialize
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("Error: DATABASE_URL is not set");
    let pool = PgPool::connect(&database_url).await?;
    database::initialize(&pool).await?;

    // parse files and add data to database
    let mut handles = Vec::new();
    for file in args.files {
        handles.push(database::insert_filename(file, pool.clone()));
    }

    // finish all jobs
    for job in handles {
        job.await?;
    }

    database::finalize(&pool).await?;
    Ok(())
}
