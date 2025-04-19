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
            CREATE TABLE euiv (
                primary_id SERIAL PRIMARY KEY,
                group_id INT REFERENCES euiv(child_id) ON DELETE RESTRICT ON UPDATE CASCADE,
                key VARCHAR(255) NOT NULL,
                value VARCHAR(255),
                parent_id INT REFERENCES euiv(primary_id) ON DELETE RESTRICT ON UPDATE CASCADE,
                child_id INT UNIQUE DEFAULT nextval('euiv_childseq')
                CONSTRAINT value_or_child_id_not_null CHECK (value IS NOT NULL OR child_id IS NOT NULL)
            )
        ",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_filename(file: String, pool: Pool<Postgres>) -> Result<(), sqlx::Error> {
    let unparsedfile = UnparsedFile::new(&file);
    let parsedfile = unparsedfile.process();
    let filename = parsedfile.filename;
    let parsed = parsedfile.parsed;

    query!("INSERT INTO euiv (key) VALUES ($1)", filename)
        .execute(&pool)
        .await?;

    let ids = query!(
        "SELECT primary_id, child_id FROM euiv WHERE key = $1",
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
            query!(
                "INSERT INTO euiv (group_id, key, parent_id) VALUES ($1, $2, $3)",
                group_id,
                key,
                parent_id
            )
            .execute(&pool)
            .await?;

            let ids = query!("SELECT primary_id, child_id FROM euiv WHERE group_id = $1 AND key = $2 AND parent_id = $3", group_id, key, parent_id)
                .fetch_one(&pool)
                .await?;

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
