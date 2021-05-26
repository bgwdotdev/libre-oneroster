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

/// Creates a database call function to the relevant json array object view
/// $name is the name of the function mirroring the HTTP API get request
/// $data is the json array struct to serialize to
/// $query is the SQL query to the relevant view
/// $object is the json object contained in the $data struct
macro_rules! create_get_db {
    ($name:ident, $data:ty, $query:literal, $object:ident) => {
        pub(crate) async fn $name(db: &sqlx::SqlitePool) -> Result<$data> {
            let row = sqlx::query!($query).fetch_one(db).await?;
            if let Some(data) = row.$object {
                let output: $data = serde_json::from_str(&data)?;
                if output.$object.len() >= 1 {
                    return Ok(output);
                }
            }
            Err(ServerError::NoContent)
        }
    };
}

create_get_db!(
    get_all_classes,
    model::Classes,
    "SELECT classes FROM ClassesJsonArray",
    classes
);
create_get_db!(
    get_all_academic_sessions,
    model::AcademicSessions,
    "SELECT academicSessions AS academic_sessions FROM AcademicSessionsJsonArray",
    academic_sessions
);
create_get_db!(
    get_all_periods,
    model::Periods,
    "SELECT periods FROM PeriodsJsonArray",
    periods
);
create_get_db!(
    get_all_orgs,
    model::Orgs,
    "SELECT orgs FROM OrgsJsonArray",
    orgs
);
create_get_db!(
    get_all_users,
    model::Users,
    "SELECT users FROM UsersJsonArray",
    users
);
create_get_db!(
    get_all_subjects,
    model::Subjects,
    "SELECT subjects FROM SubjectsJsonArray",
    subjects
);
create_get_db!(
    get_all_courses,
    model::Courses,
    "SELECT courses FROM CoursesJsonArray",
    courses
);

macro_rules! create_put_db {
    ($name:ident, $data:ty, $query:literal, $object:ident) => {
        pub(crate) async fn $name(data: $data, db: &sqlx::SqlitePool) -> Result<()> {
            let mut transaction = db.begin().await?;
            for i in data.$object.iter() {
                let json = serde_json::to_string(i)?;
                sqlx::query!($query, json).execute(&mut transaction).await?;
            }
            transaction.commit().await?;
            Ok(())
        }
    };
}

create_put_db!(
    put_academic_sessions,
    model::AcademicSessions,
    "INSERT INTO AcademicSessionsJson(academicSession) VALUES (json(?))",
    academic_sessions
);
create_put_db!(
    put_periods,
    model::Periods,
    "INSERT INTO PeriodsJson(period) VALUES (json(?))",
    periods
);
create_put_db!(
    put_subjects,
    model::Subjects,
    "INSERT INTO SubjectsJson(subject) VALUES (json(?))",
    subjects
);
create_put_db!(
    put_classes,
    model::Classes,
    "INSERT INTO ClassesJson(class) VALUES (json(?))",
    classes
);
create_put_db!(
    put_courses,
    model::Courses,
    "INSERT INTO CoursesJson(course) VALUES (json(?))",
    courses
);
create_put_db!(
    put_orgs,
    model::Orgs,
    "INSERT INTO OrgsJson(org) VALUES (json(?))",
    orgs
);
create_put_db!(
    put_users,
    model::Users,
    "INSERT INTO UsersJson(user) VALUES (json(?))",
    users
);

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
