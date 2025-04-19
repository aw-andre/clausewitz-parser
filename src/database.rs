use super::parser::*;
use pest::iterators::Pair;
use sqlx::{Pool, Postgres, query};

pub async fn initialize(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    query!("DROP TABLE IF EXISTS euiv").execute(pool).await?;
    query!("DROP SEQUENCE IF EXISTS euiv_childseq")
        .execute(pool)
        .await?;
    query!("CREATE SEQUENCE euiv_childseq")
        .execute(pool)
        .await?;
    query!(
        "
            CREATE UNLOGGED TABLE euiv (
                primary_id SERIAL PRIMARY KEY,
                group_id INT,
                key TEXT NOT NULL,
                value TEXT,
                parent_id INT,
                child_id INT DEFAULT nextval('euiv_childseq')
            )
        ",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn finalize(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    query!("CREATE INDEX group_idx ON euiv(group_id)")
        .execute(pool)
        .await?;
    query!("CREATE INDEX key_idx ON euiv(key)")
        .execute(pool)
        .await?;
    query!("CREATE INDEX parent_idx ON euiv(parent_id)")
        .execute(pool)
        .await?;
    query!("CREATE UNIQUE INDEX child_idx ON euiv(child_id)")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_filename(file: String, pool: Pool<Postgres>) -> Result<(), sqlx::Error> {
    let unparsedfile = UnparsedFile::new(&file);
    let parsedfile = unparsedfile.process();
    let filename = parsedfile.filename;
    let parsed = parsedfile.parsed;

    let ids = query!(
        "INSERT INTO euiv (key) VALUES ($1) RETURNING primary_id, child_id",
        filename
    )
    .fetch_one(&pool)
    .await?;

    let parent_id = ids.primary_id;
    let group_id = ids.child_id.unwrap();

    insert(parsed, pool.clone(), parent_id, group_id).await?;
    Ok(())
}

async fn insert(
    parsed: Pair<'_, Rule>,
    pool: Pool<Postgres>,
    parent_id: i32,
    group_id: i32,
) -> Result<(), sqlx::Error> {
    for ident in parsed.into_inner() {
        match ident.as_rule() {
            Rule::file => Box::pin(insert(ident, pool.clone(), parent_id, group_id)).await?,
            Rule::list => Box::pin(insert(ident, pool.clone(), parent_id, group_id)).await?,
            Rule::pair => Box::pin(insert_pair(ident, pool.clone(), parent_id, group_id)).await?,
            _ => (),
        }
    }
    Ok(())
}

async fn insert_pair(
    parsed: Pair<'_, Rule>,
    pool: Pool<Postgres>,
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
                "INSERT INTO euiv (group_id, key, parent_id) VALUES ($1, $2, $3) RETURNING primary_id, child_id",
                group_id,
                key,
                parent_id
            )
            .fetch_one(&pool)
            .await?;

            println!("{:#?}", ids);
            let parent_id = ids.primary_id;
            let group_id = ids.child_id.unwrap();

            insert(possible_list.unwrap(), pool.clone(), parent_id, group_id).await?;
        }

        // value is a word
        Some(value) => {
            query!("INSERT INTO euiv (group_id, key, value, parent_id, child_id) VALUES ($1, $2, $3, $4, $5)", group_id, key, value, parent_id, None::<i32>)
                .execute(&pool)
                .await?;
        }
    }
    Ok(())
}
