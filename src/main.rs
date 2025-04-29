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

    if args.initialize {
        database::initialize(pool.clone()).await?;
    }

    if args.add {
        let mut handles = Vec::new();
        for file in args.files {
            handles.push(database::insert_file(pool.clone(), file, args.game.clone()));
        }

        for job in handles {
            job.await?;
        }
    }

    if args.delete {
        database::delete_game(pool.clone(), args.game.clone()).await?;
    }

    if args.finalize {
        database::finalize(pool.clone()).await?;
    }
    Ok(())
}
