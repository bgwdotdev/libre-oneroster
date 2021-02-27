use jsonwebtoken;
use serde_json;
use sqlite::SqlitePoolOptions;
use sqlx::{migrate::MigrateDatabase, sqlite};
use std::time::SystemTime;
use tide::prelude::*;
use tide::Request;

// server
#[derive(Clone)]
struct State {
    db: sqlx::SqlitePool,
}

pub async fn run() -> tide::Result<()> {
    env_logger::init();
    let hello = "hello";
    log::info!("starting server: {}", hello);

    let path = "sqlite:db/oneroster.db";
    db_init(path).await?;
    let pool = db_connect(path).await?;
    db_init_schema(&pool).await?;

    let state = State { db: pool };
    let url_port = "localhost:8080";
    let mut srv = tide::with_state(state);
    srv.with(JwtMiddleware::new());
    log::info!("ready on: {}", url_port);
    srv.at("/academicSessions").get(get_all_academic_sessions);
    srv.at("/academicSessions").put(put_academic_sesions);
    srv.at("/login").post(login);
    srv.at("/check_token").get(check_token);
    srv.listen(url_port).await?;
    Ok(())
}

// jwt handler
#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    aud: String,
    exp: u64,
    sub: String,
    // scopes: String,
}
// scopes:
// roster-core.readonly roster.readonly roster-demographics.readonly
// resource.readonly gradebook.readonly gradebook.createput gradebook.delete

// TODO: get keys from user
lazy_static::lazy_static! {
    static ref JWT_ENCODE_KEY: jsonwebtoken::EncodingKey = {
        jsonwebtoken::EncodingKey::from_rsa_pem(include_bytes!("../certs/localhost.key.pem"))
            .expect("Problem loading private key")
    };
    // jwt crate doesn't support x509 so must extract pub key with openssl, see:
    // https://github.com/Keats/jsonwebtoken/issues/127
    static ref JWT_DECODE_KEY: jsonwebtoken::DecodingKey = {
        let cert = openssl::x509::X509::from_pem(include_bytes!("../certs/localhost.pem"))
            .expect("problem loading pub pem");
        let pem = cert.public_key().unwrap().rsa().unwrap().public_key_to_pem().unwrap();
        let pubkey = jsonwebtoken::DecodingKey::from_rsa_pem(&pem).unwrap();
        return pubkey;
    };
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Creds {
    client_id: String,
    client_secret: String,
}

async fn login(mut req: tide::Request<State>) -> tide::Result<String> {
    let creds: Creds = req.body_form().await?;
    log::info!("login attempt from: {}", creds.client_id);
    let res = sqlx::query_as!(
        Creds,
        "SELECT client_id, client_secret FROM credentials WHERE client_id = ?",
        creds.client_id
    )
    .fetch_one(&req.state().db)
    .await?;
    // TODO: use salt/hashing
    if creds.client_secret == res.client_secret {
        let token = create_token(creds.client_id).await?;
        return Ok(token);
    }
    // TODO: provide proper return response
    Ok("nope".to_string())
}

async fn create_token(id: String) -> tide::Result<String> {
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let exp = SystemTime::now().duration_since(std::time::UNIX_EPOCH)?
        + std::time::Duration::from_secs(10);
    let claims = Claims {
        aud: "localhost".to_string(),
        exp: exp.as_secs(),
        sub: id,
    };
    let token = jsonwebtoken::encode(&header, &claims, &JWT_ENCODE_KEY)?;
    log::debug!("creating token:\n{}", token);
    Ok(token)
}

async fn validate_token(token: &str) -> bool {
    log::debug!("validating token:\n{}", token);
    let val = jsonwebtoken::Validation {
        algorithms: vec![jsonwebtoken::Algorithm::RS256],
        ..Default::default()
    };
    match jsonwebtoken::decode::<Claims>(&token, &JWT_DECODE_KEY, &val) {
        Ok(t) => {
            log::debug!("validated:\n{:?}", t);
            true
        }
        Err(_) => false,
    }
}

async fn check_token(req: tide::Request<State>) -> tide::Result<String> {
    if let Some(bearer) = req.header("Authorization").and_then(|h| h.get(0)) {
        if let Some(token) = bearer.to_string().split(' ').nth(1) {
            if validate_token(token).await == true {
                return Ok("✔ Token valid\n".to_string());
            }
        }
    }
    Ok("✗ Token invalid\n".to_string())
}

// jwt middleware
struct JwtMiddleware {}

impl JwtMiddleware {
    fn new() -> Self {
        Self {}
    }
}

#[tide::utils::async_trait]
impl tide::Middleware<State> for JwtMiddleware {
    async fn handle(&self, req: tide::Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        let h = req.header("Authorization");
        log::debug!("Authorization Header:\n{:?}", h);
        if let Some(_) = h {
            let res = next.run(req).await;
            Ok(res)
        } else {
            Ok(tide::Response::new(tide::StatusCode::Unauthorized))
        }
    }
}

// endpoints
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Test {
    sourced_id: String,
    status: String,
    year: Option<String>,
}

async fn put_academic_sesions(mut req: Request<State>) -> tide::Result<String> {
    let j: Vec<Test> = req.body_json().await?;
    let mut t = req.state().db.begin().await?;
    log::debug!("put req for: {:?}", j);

    for i in j.iter() {
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

    Ok("ok \n".to_string()) // TODO: implement proper response
}

async fn get_all_academic_sessions(req: Request<State>) -> tide::Result<String> {
    let rows = sqlx::query!("SELECT json(data) as data FROM academicSessions")
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

// db
async fn db_connect(path: &str) -> sqlx::Result<sqlx::Pool<sqlx::Sqlite>> {
    log::info!("connecting to database...");
    return SqlitePoolOptions::new()
        .max_connections(1)
        .connect(path)
        .await;
}

async fn db_init_schema(pool: &sqlx::SqlitePool) -> sqlx::Result<()> {
    let mut t = pool.begin().await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS academicSessions
        (id INTEGER PRIMARY KEY AUTOINCREMENT, sourcedId STRING UNIQUE, data JSON);
        CREATE TABLE IF NOT EXISTS credentials
        (id INTEGER PRIMARY KEY AUTOINCREMENT, client_id TEXT UNIQUE NOT NULL, client_secret TEXT NOT NULL);
        "#,
    ).execute(&mut t).await?;
    t.commit().await?;
    Ok(())
}

// tests
#[cfg(test)]
#[async_std::test]
async fn db() -> sqlx::Result<()> {
    let path = "sqlite:db/rust_test.db";
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
