use serde_json;
use sqlite::{SqlitePoolOptions, SqliteQueryResult};
use sqlx::{migrate::MigrateDatabase, sqlite, Executor};
use tide::prelude::*;
use tide::Request;

#[derive(Clone)]
struct State {
    db: sqlx::SqlitePool,
}
pub async fn run() -> tide::Result<()> {
    env_logger::init();

    let path = "sqlite:oneroster.db";
    db_init(path).await?;
    let pool = db_connect(path).await?;
    db_init_schema(&pool).await?;
    get(&pool).await?;

    let state = State { db: pool };
    let mut srv = tide::with_state(state);
    srv.at("/academicSessions").get(get_all_academic_sessions);
    srv.listen("localhost:8080").await?;
    Ok(())
}

async fn get_all_academic_sessions(req: Request<State>) -> tide::Result<String> {
    let rows = sqlx::query!("SELECT data FROM academicSessions")
        .fetch_all(&req.state().db)
        .await?;

    let mut vs: Vec<serde_json::Value> = Vec::new();
    for row in rows.into_iter() {
        if let Some(d) = row.data {
            let v: serde_json::Value = serde_json::from_str(&d)?;
            &vs.push(v);
        }
    }
    let j = serde_json::to_string(&vs)?;
    Ok(format!("{} \n", j))
}

async fn get(pool: &sqlx::SqlitePool) -> tide::Result<()> {
    let rows = sqlx::query!("SELECT data FROM academicSessions")
        .fetch_all(pool)
        .await?;

    for row in rows.into_iter() {
        if let Some(d) = row.data {
            let v: serde_json::Value = serde_json::from_str(&d)?;
            println!("{:?}", v);
        }
    }

    Ok(())
}

async fn db_init(path: &str) -> sqlx::Result<()> {
    log::info!("seeking database...");
    let exist = sqlx::Sqlite::database_exists(path).await?;
    if exist {
        log::info!("found existing database");
    } else {
        log::info!("no existing database, creating...");
        sqlx::Sqlite::create_database(path).await?;
    };
    Ok(())
}

async fn db_connect(path: &str) -> sqlx::Result<sqlx::Pool<sqlx::Sqlite>> {
    log::info!("connecting to database...");
    return SqlitePoolOptions::new()
        .max_connections(1)
        .connect(path) // TODO: move to cmd flag
        .await;
}

async fn db_init_schema(pool: &sqlx::SqlitePool) -> sqlx::Result<()> {
    let mut t = pool.begin().await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS academicSessions (id INTEGER PRIMARY KEY AUTOINCREMENT, sourcedId STRING UNIQUE, data JSON)",
    ).execute(&mut t).await?;
    t.commit().await?;
    Ok(())
}

#[cfg(test)]
#[async_std::test]
async fn db() -> sqlx::Result<()> {
    let path = "sqlite:rust_test.db";
    db_init(path).await?;
    let pool = db_connect(path).await?;
    db_init_schema(&pool).await?;

    sqlx::query(
        r#"INSERT INTO academicSessions (sourcedId, data) values (
            43278488,
            json('{
                "sourcedId" : "43278488",
                "status" : "active"
            }')
        ) ON CONFLICT(sourcedId) DO UPDATE SET data=excluded.data"#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"INSERT INTO academicSessions (sourcedId, data) values (
            43278489,
            json('{
                "sourcedId" : "43278489",
                "status" : "tobedeleted"
            }')
        ) ON CONFLICT(sourcedId) DO UPDATE SET data=excluded.data"#,
    )
    .execute(&pool)
    .await?;

    Ok(())
}
