mod auth;
mod db;
pub mod errors;
mod params;

use crate::model;
pub use errors::*;
use http_types::mime;
use std::fs::File;
use std::io::prelude::*;
use tide::prelude::*;
use tide::utils::After;
use tide::Request;

type Result<T> = std::result::Result<T, ServerError>;

#[derive(Clone)]
pub(crate) struct State {
    db: sqlx::SqlitePool,
    encode_key: jsonwebtoken::EncodingKey,
    decode_key: jsonwebtoken::DecodingKey,
}

/// Creates a GET endpoint function
/// $name takes the name of the function to generate as well as the matching DB req function
/// $object takes the name of the top level json object within the collection { "myObject": [{}] }
/// $wrapper takes the name of the top level json object as a string for JQ to use in querying
macro_rules! create_get_endpoint {
    ($name:ident, $object:ident, $wrapper:literal) => {
        async fn $name(req: Request<State>) -> tide::Result {
            let params = req.query()?;
            let data = db::$name(&req.state().db).await?;
            let links = params::link_header_builder(&req, &params, data.$object.len()).await;
            let (output, total) =
                params::apply_parameters(&json!(data).to_string(), &params, $wrapper).await?;
            Ok(tide::Response::builder(200)
                .header("link", links)
                .header("x-total-count", total.trim())
                .content_type(mime::JSON)
                .body(output)
                .build())
        }
    };
}

create_get_endpoint!(get_all_classes, classes, "classes");
create_get_endpoint!(
    get_all_academic_sessions,
    academic_sessions,
    "academicSessions"
);
create_get_endpoint!(get_all_periods, periods, "periods");
create_get_endpoint!(get_all_orgs, orgs, "orgs");
create_get_endpoint!(get_all_users, users, "users");
create_get_endpoint!(get_all_subjects, subjects, "subjects");
create_get_endpoint!(get_all_courses, courses, "courses");
create_get_endpoint!(get_all_enrollments, enrollments, "enrollments");
// enrollment

macro_rules! create_put_endpoint {
    ($i:ident) => {
        async fn $i(mut req: Request<State>) -> tide::Result {
            let json = to_vec(&mut req).await?;
            log::debug!("put request for: {:?}", json);
            db::$i(json, &req.state().db).await?;
            Ok(tide::Response::builder(200).build())
        }
    };
}

create_put_endpoint!(put_academic_sessions);
create_put_endpoint!(put_periods);
create_put_endpoint!(put_orgs);
create_put_endpoint!(put_users);
create_put_endpoint!(put_subjects);
create_put_endpoint!(put_courses);
create_put_endpoint!(put_classes);
create_put_endpoint!(put_enrollments);

#[derive(Debug)]
pub struct Config {
    pub database: String,
    pub init: bool,
    pub socket_address: std::net::SocketAddr,
    pub encode_key: jsonwebtoken::EncodingKey,
    pub decode_key: jsonwebtoken::DecodingKey,
}

pub async fn run(config: Config) -> tide::Result<()> {
    log::info!("starting server...");
    log::debug!("configuration: {:?}", config);

    let path = "sqlite:".to_owned() + &config.database;
    let pool = match db::init(&path, config.init).await {
        Ok(pool) => pool,
        Err(e) => {
            log::error!("Error: could not start server: {}", e);
            return Ok(());
        }
    };

    let state = State {
        db: pool,
        encode_key: config.encode_key,
        decode_key: config.decode_key,
    };
    let mut srv = tide::with_state(state);

    srv.with(After(errors::middleware::ApiError::new()));
    log::info!("ready on: {}", &config.socket_address);
    srv.at("/").get(|_| async { Ok("oneroster ui\n") });
    srv.at("/auth/login").post(login);
    srv.at("/auth/check_token").get(check_token);
    // oneroster
    let mut authsrv = tide::with_state(srv.state().clone());
    authsrv.with(auth::middleware::Jwt::new(vec![
        "roster-core".to_string(),
        "roster".to_string(),
    ]));
    authsrv
        .at("/")
        .get(|_| async { Ok("hello protected world\n") });
    authsrv.at("/orgs").get(get_all_orgs).put(put_orgs);
    authsrv.at("/classes").get(get_all_classes).put(put_classes);
    authsrv
        .at("/academicSessions")
        .get(get_all_academic_sessions)
        .put(put_academic_sessions);
    authsrv.at("/periods").get(get_all_periods).put(put_periods);
    authsrv
        .at("/subjects")
        .get(get_all_subjects)
        .put(put_subjects);
    authsrv.at("/courses").get(get_all_courses).put(put_courses);
    authsrv.at("/users").get(get_all_users).put(put_users);
    authsrv
        .at("/enrollments")
        .get(get_all_enrollments)
        .put(put_enrollments);
    // user management
    let mut adminsrv = tide::with_state(srv.state().clone());
    adminsrv.with(auth::middleware::Jwt::new(vec!["admin".to_string()]));
    adminsrv.at("/users").get(get_api_users);
    adminsrv.at("/user").post(create_api_user);
    adminsrv.at("/user/:uuid").delete(delete_api_user);

    srv.at("/admin").nest(adminsrv);
    srv.at("/ims/oneroster/v1p1").nest(authsrv);
    srv.listen(config.socket_address).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
//#[serde(rename_all = "camelCase")]
pub struct Creds {
    client_id: String,
    client_secret: String,
    scope: String,
}

//async fn to_vec<T>(req: &mut Request<State>) -> Result<Vec<T>>
async fn to_vec<T>(req: &mut Request<State>) -> Result<T>
where
    for<'a> T: Deserialize<'a>,
{
    let s = req.body_string().await.unwrap();
    let v = serde_json::from_str(&s)?;
    Ok(v)
}

async fn login(mut req: tide::Request<State>) -> tide::Result {
    log::debug!("login request");
    let creds: Creds = req.body_form().await?;
    log::info!("login attempt from: {}", creds.client_id);
    let token = auth::credentials::login(creds, &req.state().db, &req.state().encode_key).await?;
    Ok(tide::Response::builder(200).body(json!(token)).build())
}

async fn create_api_user(mut req: tide::Request<State>) -> tide::Result {
    let new: db::CreateApiUser = req.body_json().await?;
    let creds = db::create_api_user(new, &req.state().db).await?;
    Ok(tide::Response::builder(200).body(json!(creds)).build())
}

async fn delete_api_user(req: tide::Request<State>) -> tide::Result {
    let uuid = req.param("uuid")?;
    db::delete_api_user(uuid, &req.state().db).await?;
    Ok(tide::Response::builder(200).build())
}

async fn get_api_users(req: tide::Request<State>) -> tide::Result {
    let res = db::get_api_users(&req.state().db).await?;
    Ok(tide::Response::builder(200).body(json!(res)).build())
}

async fn check_token(req: tide::Request<State>) -> tide::Result<String> {
    let token = auth::middleware::parse_auth_header(&req).await?;
    if auth::jwt::validate_token(token, &req.state().decode_key).await {
        return Ok("✔ Token valid\n".to_string());
    }
    Ok("✗ Token invalid\n".to_string())
}

pub fn read_private_key(path: &str) -> Result<jsonwebtoken::EncodingKey> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let private_key = jsonwebtoken::EncodingKey::from_rsa_pem(&buf)?;
    Ok(private_key)
}

pub fn read_public_key(path: &str) -> Result<jsonwebtoken::DecodingKey> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let cert = openssl::x509::X509::from_pem(&buf)?;
    let pem = cert.public_key()?.rsa()?.public_key_to_pem()?;
    let public_key = jsonwebtoken::DecodingKey::from_rsa_pem(&pem)?;
    Ok(public_key)
}

// tests
#[cfg(test)]
#[async_std::test]
async fn db() -> Result<()> {
    let path = "sqlite:db/rust_test.db";
    db::init(path, true).await?;
    let pool = db::connect(path).await?;

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
