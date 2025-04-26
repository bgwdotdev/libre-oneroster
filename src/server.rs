mod auth;
mod db;
pub mod errors;
mod params;

use async_std::prelude::*;
pub use errors::*;
use http_types::mime;
use std::io::prelude::*;
use tide::prelude::*;
use tide::utils::After;
use tide::Request;
use tide_rustls::TlsListener;

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
create_get_endpoint!(
    get_all_grading_periods,
    academic_sessions,
    "academicSessions"
);
create_get_endpoint!(get_all_schools, orgs, "orgs");
create_get_endpoint!(get_all_students, users, "users");
create_get_endpoint!(get_all_teachers, users, "users");
create_get_endpoint!(get_all_terms, academic_sessions, "academicSessions");

macro_rules! create_get_endpoint_by_id {
    ($name:ident) => {
        async fn $name(req: Request<State>) -> tide::Result {
            let id = req.param("id")?;
            let data = db::$name(&req.state().db, id).await?;
            Ok(tide::Response::builder(200)
                .content_type(mime::JSON)
                .header("x-total-count", "1")
                .body(json!(data).to_string())
                .build())
        }
    };
}
create_get_endpoint_by_id!(get_academic_session);
create_get_endpoint_by_id!(get_class);
create_get_endpoint_by_id!(get_course);
create_get_endpoint_by_id!(get_grading_period);
create_get_endpoint_by_id!(get_enrollment);
create_get_endpoint_by_id!(get_org);
create_get_endpoint_by_id!(get_school);
create_get_endpoint_by_id!(get_student);
create_get_endpoint_by_id!(get_teacher);
create_get_endpoint_by_id!(get_term);
create_get_endpoint_by_id!(get_user);

macro_rules! create_get_collection_endpoint_by_id {
    ($name:ident, $object:ident, $wrapper:literal) => {
        async fn $name(req: Request<State>) -> tide::Result {
            let id = req.param("id")?;
            let params = req.query()?;
            let data = db::$name(&req.state().db, &id).await?;
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

create_get_collection_endpoint_by_id!(get_classes_for_school, classes, "classes");
create_get_collection_endpoint_by_id!(get_students_for_school, users, "users");
create_get_collection_endpoint_by_id!(get_teachers_for_school, users, "users");
create_get_collection_endpoint_by_id!(get_enrollments_for_school, enrollments, "enrollments");

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

pub struct Config {
    pub database: String,
    pub init: bool,
    pub socket_address: std::net::SocketAddr,
    pub encode_key: jsonwebtoken::EncodingKey,
    pub decode_key: jsonwebtoken::DecodingKey,
    pub web_public_key: String,
    pub web_private_key: String,
}

pub async fn run(config: Config) -> tide::Result<()> {
    log::info!("starting server...");
    //log::debug!("configuration: {:?}", config);

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
    authsrv.at("/orgs/:id").get(get_org);
    authsrv.at("/schools").get(get_all_schools);
    authsrv.at("/schools/:id").get(get_school);
    authsrv
        .at("/schools/:id/classes")
        .get(get_classes_for_school);
    authsrv
        .at("/schools/:id/students")
        .get(get_students_for_school);
    authsrv
        .at("/schools/:id/teachers")
        .get(get_teachers_for_school);
    authsrv
        .at("/schools/:id/enrollments")
        .get(get_enrollments_for_school);
    authsrv.at("/classes").get(get_all_classes).put(put_classes);
    authsrv.at("/classes/:id").get(get_class);
    authsrv
        .at("/academicSessions")
        .get(get_all_academic_sessions)
        .put(put_academic_sessions);
    authsrv
        .at("/academicSessions/:id")
        .get(get_academic_session);
    authsrv.at("/gradingPeriods").get(get_all_grading_periods);
    authsrv.at("/gradingPeriods/:id").get(get_grading_period);
    authsrv.at("/periods").get(get_all_periods).put(put_periods);
    authsrv
        .at("/subjects")
        .get(get_all_subjects)
        .put(put_subjects);
    authsrv.at("/courses").get(get_all_courses).put(put_courses);
    authsrv.at("/courses/:id").get(get_course);
    authsrv.at("/users").get(get_all_users).put(put_users);
    authsrv.at("/users/:id").get(get_user);
    authsrv.at("/students").get(get_all_students);
    authsrv.at("/students/:id").get(get_student);
    authsrv.at("/teachers").get(get_all_teachers);
    authsrv.at("/teachers/:id").get(get_teacher);
    authsrv.at("/terms").get(get_all_terms);
    authsrv.at("/terms/:id").get(get_term);
    authsrv
        .at("/enrollments")
        .get(get_all_enrollments)
        .put(put_enrollments);
    authsrv.at("/enrollments/:id").get(get_enrollment);
    // user management
    let mut adminsrv = tide::with_state(srv.state().clone());
    adminsrv.with(auth::middleware::Jwt::new(vec!["admin".to_string()]));
    adminsrv.at("/users").get(get_api_users);
    adminsrv.at("/user").post(create_api_user);
    adminsrv.at("/user/:uuid").delete(delete_api_user);

    srv.at("/admin").nest(adminsrv);
    srv.at("/ims/oneroster/v1p1").nest(authsrv);
    srv.listen(
        TlsListener::build()
            .addrs(config.socket_address)
            .cert(config.web_public_key)
            .key(config.web_private_key),
        //config.socket_address
    )
    .await?;
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
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let private_key = jsonwebtoken::EncodingKey::from_rsa_pem(&buf)?;
    Ok(private_key)
}

pub fn read_public_key(path: &str) -> Result<jsonwebtoken::DecodingKey> {
    let mut file = std::fs::File::open(path)?;
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
    let path = "sqlite:./db/rust_test.db";
    db::init(path, true).await?;
    let pool = db::connect(path).await?;
    let content = async_std::fs::read_to_string("./sample/academicSessions.json").await?;
    let json = serde_json::from_str(&content)?;
    db::put_academic_sessions(json, &pool).await?;
    Ok(())
}
