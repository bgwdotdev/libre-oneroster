use crate::model;
use crate::server::{auth, Result, ServerError};
use model::Org;
use sqlite::SqlitePoolOptions;
use sqlx::{migrate::MigrateDatabase, sqlite};
use tide::prelude::*;

#[derive(Serialize)]
pub(super) struct UserList {
    tag: String,
    client_id: String,
    scope: String,
}

pub(super) async fn get_api_creds(
    client_id: &String,
    db: &sqlx::SqlitePool,
) -> Result<super::Creds> {
    let res = sqlx::query_as!(
        super::Creds,
        r#"
        SELECT
            c.client_id
            , c.client_secret
            , group_concat(s.scope,' ') AS "scope!: String"
        FROM
            credentials c
            INNER JOIN credential_scopes cs ON c.id = cs.credential_id
            INNER JOIN scopes s ON cs.scope_id = s.id
        WHERE
            c.client_id = ?
            AND scope IS NOT NULL
        GROUP BY 
            c.client_id
        "#,
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
) -> Result<Vec<UserList>> {
    let rows = sqlx::query_as!(
        UserList,
        r#"
        SELECT
            c.client_id
            , c.tag
            , group_concat(s.scope,' ') AS "scope!: String"
        FROM
            credentials c
            INNER JOIN credential_scopes cs ON c.id = cs.credential_id
            INNER JOIN scopes s ON cs.scope_id = s.id
        WHERE
            ? = ?
            AND scope IS NOT NULL
        GROUP BY 
            c.client_id
        "#,
        fcol,
        fval,
    )
    .fetch_all(db)
    .await?;

    Ok(rows)
}

#[derive(Deserialize)]
pub(super) struct CreateApiUser {
    tag: String,
    scope: String,
}

pub(super) async fn create_api_user(
    user: CreateApiUser,
    db: &sqlx::SqlitePool,
) -> Result<super::Creds> {
    let new = auth::credentials::generate_credentials().await?;
    let mut t = db.begin().await?;
    sqlx::query!(
        "INSERT INTO credentials(client_id, client_secret, tag) VALUES (?, ?, ?)",
        new.creds.client_id,
        new.encrypt,
        user.tag,
    )
    .execute(&mut t)
    .await?;
    for scope in user.scope.split(' ') {
        sqlx::query!(
            "INSERT OR IGNORE INTO credential_scopes (credential_id, scope_id) VALUES (
            (SELECT id FROM credentials WHERE client_id = ?),
            (SELECT id FROM scopes WHERE scope = ?)
            )",
            new.creds.client_id,
            scope,
        )
        .execute(&mut t)
        .await?;
    }
    t.commit().await?;
    let authscopes = get_api_creds(&new.creds.client_id, db).await?;
    let out = super::Creds {
        client_id: new.creds.client_id.clone(),
        client_secret: new.creds.client_secret.clone(),
        scope: authscopes.scope,
    };
    Ok(out)
}

pub(super) async fn delete_api_user(uuid: &str, db: &sqlx::SqlitePool) -> Result<()> {
    let deleted = sqlx::query!("DELETE FROM credentials WHERE client_id = ?", uuid)
        .execute(db)
        .await?
        .rows_affected();

    if deleted > 0 {
        return Ok(());
    }
    Err(ServerError::NoRecordDeleted)
}

pub(crate) async fn get_all_academic_sessions(
    db: &sqlx::SqlitePool,
) -> Result<Vec<model::AcademicSession>> {
    let rows = sqlx::query!(
        r#"SELECT academicSession AS "academicSession!: String" FROM AcademicSessionsJson"#
    )
    .fetch_all(db)
    .await?;
    let ac: Result<Vec<model::AcademicSession>> = rows
        .iter()
        .map(|r| Ok(serde_json::from_str(&r.academicSession)?))
        .collect();
    Ok(ac?)
}

pub(crate) async fn put_academic_sessions(
    data: Vec<model::AcademicSession>,
    db: &sqlx::SqlitePool,
) -> Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let j = serde_json::to_string(i)?;
        sqlx::query!(
            r#"INSERT INTO AcademicSessionsJson(academicSession) VALUES (json(?))"#,
            j
        )
        .execute(&mut t)
        .await?;
    }
    t.commit().await?;
    Ok(())
}

pub(super) async fn get_all_periods(db: &sqlx::SqlitePool) -> Result<Vec<model::Period>> {
    let rows = sqlx::query!(r#"SELECT period AS "period!: String" FROM PeriodsJson"#)
        .fetch_all(db)
        .await?;
    let period: Result<Vec<model::Period>> = rows
        .iter()
        .map(|r| Ok(serde_json::from_str(&r.period)?))
        .collect();
    Ok(period?)
}

pub(super) async fn put_periods(data: Vec<model::Period>, db: &sqlx::SqlitePool) -> Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let j = serde_json::to_string(i)?;
        sqlx::query!(r#"INSERT INTO PeriodsJson(period) VALUES (json(?))"#, j)
            .execute(&mut t)
            .await?;
    }
    t.commit().await?;
    Ok(())
}

pub(super) async fn get_all_subjects(db: &sqlx::SqlitePool) -> Result<Vec<model::Subject>> {
    let rows = sqlx::query!(r#"SELECT subject AS "subject!: String" FROM SubjectsJson"#)
        .fetch_all(db)
        .await?;
    let subject: Result<Vec<model::Subject>> = rows
        .iter()
        .map(|r| Ok(serde_json::from_str(&r.subject)?))
        .collect();
    Ok(subject?)
}

pub(super) async fn put_subjects(data: Vec<model::Subject>, db: &sqlx::SqlitePool) -> Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let j = serde_json::to_string(i)?;
        sqlx::query!(r#"INSERT INTO SubjectsJson(subject) VALUES (json(?))"#, j)
            .execute(&mut t)
            .await?;
    }
    t.commit().await?;
    Ok(())
}

pub(super) async fn get_all_classes(db: &sqlx::SqlitePool) -> Result<Vec<model::Class>> {
    let rows = sqlx::query!(r#"SELECT class AS "class!: String" FROM ClassesJson"#)
        .fetch_all(db)
        .await?;
    let classes: Result<Vec<model::Class>> = rows
        .iter()
        .map(|r| Ok(serde_json::from_str(&r.class)?))
        .collect();
    Ok(classes?)
}

pub(super) async fn put_classes(data: Vec<model::Class>, db: &sqlx::SqlitePool) -> Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let j = serde_json::to_string(i)?;
        sqlx::query!(r#"INSERT INTO ClassesJson(class) VALUES (json(?))"#, j)
            .execute(&mut t)
            .await?;
    }
    t.commit().await?;
    Ok(())
}

pub(super) async fn get_all_courses(db: &sqlx::SqlitePool) -> Result<Vec<model::Course>> {
    let rows = sqlx::query!(r#"SELECT course AS "course!: String" FROM CoursesJson"#)
        .fetch_all(db)
        .await?;
    let course: Result<Vec<model::Course>> = rows
        .iter()
        .map(|r| Ok(serde_json::from_str(&r.course)?))
        .collect();
    Ok(course?)
}

pub(super) async fn put_courses(data: Vec<model::Course>, db: &sqlx::SqlitePool) -> Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let j = serde_json::to_string(i)?;
        sqlx::query!(r#"INSERT INTO CoursesJson(course) VALUES (json(?))"#, j)
            .execute(&mut t)
            .await?;
    }
    t.commit().await?;
    Ok(())
}

pub(super) async fn get_all_orgs(db: &sqlx::SqlitePool) -> Result<Vec<model::Org>> {
    let rows = sqlx::query!(r#"SELECT org AS "org!: String" FROM OrgsJson"#)
        .fetch_all(db)
        .await?;
    let orgs: Result<Vec<Org>> = rows
        .iter()
        .map(|r| Ok(serde_json::from_str(&r.org)?))
        .collect();
    Ok(orgs?)
}

pub(super) async fn put_orgs(data: Vec<model::Org>, db: &sqlx::SqlitePool) -> Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let j = serde_json::to_string(i)?;
        sqlx::query!(r#"INSERT INTO OrgsJson(org) VALUES (json(?))"#, j)
            .execute(&mut t)
            .await?;
    }
    t.commit().await?;

    Ok(())
}

pub(super) async fn get_all_users(db: &sqlx::SqlitePool) -> Result<Vec<model::User>> {
    let rows = sqlx::query!(r#"SELECT user AS "user!: String" FROM UsersJson"#)
        .fetch_all(db)
        .await?;
    let users: Result<Vec<model::User>> = rows
        .iter()
        .map(|r| Ok(serde_json::from_str(&r.user)?))
        .collect();
    Ok(users?)
}

pub(super) async fn put_users(data: Vec<model::User>, db: &sqlx::SqlitePool) -> Result<()> {
    let mut t = db.begin().await?;
    for i in data.iter() {
        let j = serde_json::to_string(i)?;
        sqlx::query!(r#"INSERT INTO UsersJson(user) VALUES (json(?))"#, j)
            .execute(&mut t)
            .await?;
    }
    t.commit().await?;

    Ok(())
}

pub(super) async fn init(path: &str) -> Result<sqlx::Pool<sqlx::Sqlite>> {
    init_db(path).await?;
    let pool = connect(path).await?;
    init_schema(&pool).await?;
    Ok(pool)
}

pub(super) async fn init_db(path: &str) -> Result<()> {
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

pub(super) async fn init_schema(pool: &sqlx::SqlitePool) -> Result<()> {
    let mut t = pool.begin().await?;
    sqlx::query_file!("db/schema.sql").execute(&mut t).await?;
    sqlx::query_file!("db/init.sql").execute(&mut t).await?;
    t.commit().await?;
    Ok(())
}

pub(super) async fn connect(path: &str) -> Result<sqlx::Pool<sqlx::Sqlite>> {
    log::info!("connecting to database...");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(path)
        .await?;
    Ok(pool)
}
