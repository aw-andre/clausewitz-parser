use super::parser::*;
use pest::iterators::Pair;
use sqlx::{PgConnection, Pool, Postgres, query};

pub async fn initialize(pool: Pool<Postgres>) -> Result<(), sqlx::Error> {
    query!("DROP TABLE IF EXISTS gamefiles")
        .execute(&pool)
        .await?;
    query!("DROP SEQUENCE IF EXISTS gamefiles_childseq")
        .execute(&pool)
        .await?;
    query!("CREATE SEQUENCE gamefiles_childseq")
        .execute(&pool)
        .await?;
    query!(
        "
            CREATE UNLOGGED TABLE gamefiles (
                game TEXT NOT NULL,
                primary_id SERIAL PRIMARY KEY,
                group_id INT,
                key TEXT NOT NULL,
                value TEXT,
                parent_id INT,
                child_id INT DEFAULT nextval('gamefiles_childseq')
            )
        ",
    )
    .execute(&pool)
    .await?;
    Ok(())
}

pub async fn finalize(pool: Pool<Postgres>) -> Result<(), sqlx::Error> {
    query!("CREATE INDEX IF NOT EXISTS game_idx ON gamefiles(game)")
        .execute(&pool)
        .await?;
    query!("CREATE INDEX IF NOT EXISTS group_idx ON gamefiles(group_id)")
        .execute(&pool)
        .await?;
    query!("CREATE INDEX IF NOT EXISTS key_idx ON gamefiles(game, key)")
        .execute(&pool)
        .await?;
    query!("CREATE INDEX IF NOT EXISTS value_idx ON gamefiles(game, value)")
        .execute(&pool)
        .await?;
    query!("CREATE INDEX IF NOT EXISTS parent_idx ON gamefiles(parent_id)")
        .execute(&pool)
        .await?;
    query!("CREATE UNIQUE INDEX IF NOT EXISTS child_idx ON gamefiles(child_id)")
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn drop_indices(pool: &mut PgConnection) -> Result<(), sqlx::Error> {
    query!("DROP INDEX IF EXISTS game_idx")
        .execute(&mut *pool)
        .await?;
    query!("DROP INDEX IF EXISTS group_idx")
        .execute(&mut *pool)
        .await?;
    query!("DROP INDEX IF EXISTS key_idx")
        .execute(&mut *pool)
        .await?;
    query!("DROP INDEX IF EXISTS value_idx")
        .execute(&mut *pool)
        .await?;
    query!("DROP INDEX IF EXISTS parent_idx")
        .execute(&mut *pool)
        .await?;
    query!("DROP INDEX IF EXISTS child_idx")
        .execute(&mut *pool)
        .await?;
    Ok(())
}

pub async fn insert_file(
    pool: &mut PgConnection,
    file: String,
    game: String,
) -> Result<(), sqlx::Error> {
    let unparsedfile = UnparsedFile::new(&file);
    let parsedfile = unparsedfile.process();
    let filename = parsedfile.filename;
    let parsed = parsedfile.parsed;

    let ids = query!(
        "INSERT INTO gamefiles (game, key) VALUES ($1, $2) RETURNING primary_id, child_id",
        game,
        filename
    )
    .fetch_one(&mut *pool)
    .await?;

    let primary_id = ids.primary_id;
    let child_id = ids.child_id.unwrap();

    insert(parsed, &mut *pool, game, primary_id, child_id).await?;
    println!("finished inserting: {}", filename);
    println!("id: {}", primary_id);
    Ok(())
}

async fn insert(
    parsed: Pair<'_, Rule>,
    pool: &mut PgConnection,
    game: String,
    parent_id: i32,
    group_id: i32,
) -> Result<(), sqlx::Error> {
    for ident in parsed.into_inner() {
        match ident.as_rule() {
            Rule::file => {
                Box::pin(insert(ident, &mut *pool, game.clone(), parent_id, group_id)).await?
            }
            Rule::list => {
                Box::pin(insert(ident, &mut *pool, game.clone(), parent_id, group_id)).await?
            }
            Rule::pair => {
                Box::pin(insert_pair(
                    ident,
                    &mut *pool,
                    game.clone(),
                    parent_id,
                    group_id,
                ))
                .await?
            }
            _ => (),
        }
    }
    Ok(())
}

async fn insert_pair(
    parsed: Pair<'_, Rule>,
    pool: &mut PgConnection,
    game: String,
    parent_id: i32,
    group_id: i32,
) -> Result<(), sqlx::Error> {
    let mut key = "";
    let mut possible_value = None;
    let mut possible_list = None;
    for ident in parsed.into_inner() {
        match ident.as_rule() {
            Rule::key => key = ident.as_str(),
            Rule::value => {
                for inner in ident.into_inner() {
                    match inner.as_rule() {
                        Rule::word => possible_value = Some(inner.as_str()),
                        Rule::list => possible_list = Some(inner),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    match possible_value {
        // value is a list
        None => {
            let ids = query!(
                "INSERT INTO gamefiles (game, group_id, key, parent_id) VALUES ($1, $2, $3, $4) RETURNING primary_id, child_id",
                game,
                group_id,
                key,
                parent_id
            )
            .fetch_one(&mut *pool)
            .await?;

            let parent_id = ids.primary_id;
            let group_id = ids.child_id.unwrap();

            insert(
                possible_list.unwrap(),
                &mut *pool,
                game.clone(),
                parent_id,
                group_id,
            )
            .await?;
        }

        // value is a word
        Some(value) => {
            query!("INSERT INTO gamefiles (game, group_id, key, value, parent_id, child_id) VALUES ($1, $2, $3, $4, $5, $6)", game, group_id, key, value, parent_id, None::<i32>)
                .execute(&mut *pool)
                .await?;
        }
    }
    Ok(())
}

pub async fn delete_game(pool: Pool<Postgres>, game: String) -> Result<(), sqlx::Error> {
    query!("DELETE FROM gamefiles WHERE game = $1", game)
        .execute(&pool)
        .await?;
    Ok(())
}
