use crate::model;
use crate::server::auth;
use sqlite::SqlitePoolOptions;
use sqlx::{migrate::MigrateDatabase, sqlite};
use tide::prelude::*;

#[derive(Serialize)]
pub(super) struct UserList {
    tag: String,
    client_id: String,
}

pub(super) async fn get_api_creds(
    client_id: &String,
    db: &sqlx::SqlitePool,
) -> sqlx::Result<super::Creds> {
    let res = sqlx::query_as!(
        super::Creds,
        "SELECT client_id, client_secret FROM credentials WHERE client_id = ?",
        client_id
    )
    .fetch_one(db)
    .await?;
    Ok(res)
}

pub(super) async fn get_api_users(
    fcol: String,
    fval: String,
    db: &sqlx::SqlitePool,
) -> sqlx::Result<Vec<UserList>> {
    let rows = sqlx::query_as!(
        UserList,
        "SELECT tag, client_id FROM credentials WHERE ? = ?",
        fcol,
        fval,
    )
    .fetch_all(db)
    .await?;

    Ok(rows)
}

pub(crate) async fn create_api_user(
    tag: &str,
    db: &sqlx::SqlitePool,
) -> tide::Result<super::Creds> {
    let new = auth::generate_credentials().await?;
    sqlx::query!(
        "INSERT INTO credentials(client_id, client_secret, tag) VALUES (?, ?, ?)",
        new.creds.client_id,
        new.encrypt,
        tag
    )
    .execute(db)
    .await?;
    Ok(new.creds)
}

pub(super) async fn delete_api_user(uuid: &str, db: &sqlx::SqlitePool) -> sqlx::Result<bool> {
    let deleted = sqlx::query!("DELETE FROM credentials WHERE client_id = ?", uuid)
        .execute(db)
        .await?
        .rows_affected();

    if deleted > 0 {
        return Ok(true);
    }
    Ok(false)
}

pub(crate) async fn get_all_academic_sessions(
    db: &sqlx::SqlitePool,
) -> sqlx::Result<Vec<model::AcademicSession>> {
    let rows = sqlx::query!("SELECT json(data) as data FROM academicSessions")
        .fetch_all(db)
        .await?;
    let mut vs: Vec<model::AcademicSession> = Vec::new();
    for row in rows.into_iter() {
        if let Some(d) = row.data {
            let v = serde_json::from_str(&d).unwrap(); // TODO: custom error handler
            &vs.push(v);
        }
    }
    Ok(vs)
}

pub(crate) async fn put_academic_sessions(
    data: Vec<model::AcademicSession>,
    db: &sqlx::SqlitePool,
) -> sqlx::Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let json = json!(i).to_string();
        sqlx::query!(
            r#"INSERT INTO academicSessions(sourcedId, data)
            VALUES(?, json(?))
            ON CONFLICT(sourcedId)
            DO UPDATE SET sourcedId=excluded.sourcedId, data=excluded.data"#,
            i.sourced_id,
            json,
        )
        .execute(&mut t)
        .await?;
    }
    t.commit().await?;
    Ok(())
}

pub(super) async fn init(path: &str) -> sqlx::Result<()> {
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

pub(super) async fn init_schema(pool: &sqlx::SqlitePool) -> sqlx::Result<()> {
    let mut t = pool.begin().await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS academicSessions
        (id INTEGER PRIMARY KEY AUTOINCREMENT, sourcedId STRING UNIQUE, data JSON);

        CREATE TABLE IF NOT EXISTS credentials
        (id INTEGER PRIMARY KEY AUTOINCREMENT, client_id TEXT UNIQUE NOT NULL,
        client_secret TEXT NOT NULL, tag TEXT NOT NULL);

        "#,
    )
    .execute(&mut t)
    .await?;
    t.commit().await?;
    Ok(())
}

pub(super) async fn connect(path: &str) -> sqlx::Result<sqlx::Pool<sqlx::Sqlite>> {
    log::info!("connecting to database...");
    return SqlitePoolOptions::new()
        .max_connections(1)
        .connect(path)
        .await;
}
