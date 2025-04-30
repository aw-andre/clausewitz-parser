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

    if args.delete {
        database::delete_game(pool.clone(), args.game.clone().unwrap()).await?;
    }

    if args.add {
        let mut tx = pool.begin().await?;

        database::drop_indices(&mut tx).await?;

        for file in args.files {
            database::insert_file(&mut tx, file, args.game.clone().unwrap()).await?;
        }

        tx.commit().await?;
    }

    if args.finalize {
        database::finalize(pool.clone()).await?;
    }
    Ok(())
}
