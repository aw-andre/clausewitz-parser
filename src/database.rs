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
            Rule::quotechar | Rule::commentchar | Rule::wordchar | Rule::whitespace | Rule::EOI => {
                ()
            }
            Rule::file => insert(ident, pool.clone()).await?,
            Rule::list => insert_list(ident, pool.clone()).await?,
            Rule::pair => insert_pair(ident, pool.clone()).await?,
            Rule::value => insert_value(ident, pool.clone()).await?,
            Rule::key => insert_key(ident, pool.clone()).await?,
            Rule::word => insert_word(ident, pool.clone()).await?,
            Rule::COMMENT => insert_COMMENT(ident, pool.clone()).await?,
        }
    }
    Ok(())
}
