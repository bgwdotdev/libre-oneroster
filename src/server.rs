mod auth;
mod db;
mod errors;
mod params;

use crate::model;
use errors::*;
use http_types::mime;
use tide::prelude::*;
use tide::utils::After;
use tide::Request;

type Result<T> = std::result::Result<T, ServerError>;

#[derive(Clone)]
pub(crate) struct State {
    db: sqlx::SqlitePool,
}

macro_rules! create_get_endpoint {
    ($i:ident) => {
        async fn $i(req: Request<State>) -> tide::Result {
            let params = req.query()?;
            let data = db::$i(&req.state().db).await?;
            let links = params::link_header_builder(&req, &params, data.len()).await;
            let output = params::apply_parameters(&json!(data).to_string(), &params).await?;
            Ok(tide::Response::builder(200)
                .header("link", links)
                .content_type(mime::JSON)
                .body(output)
                .build())
        }
    };
}

create_get_endpoint!(get_all_academic_sessions);
create_get_endpoint!(get_all_periods);
create_get_endpoint!(get_all_orgs);
create_get_endpoint!(get_all_users);
create_get_endpoint!(get_all_subjects);
create_get_endpoint!(get_all_courses);
create_get_endpoint!(get_all_classes);
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

pub async fn run() -> tide::Result<()> {
    env_logger::init();
    let hello = "hello";
    log::info!("starting server: {}", hello);

    let path = "sqlite:db/oneroster.db";
    let pool = match db::init(path).await {
        Ok(pool) => pool,
        Err(e) => {
            log::error!("Error: could not start server: {}", e);
            return Ok(());
        }
    };

    let state = State { db: pool };
    let url_port = "localhost:8080";
    let mut srv = tide::with_state(state);

    srv.with(After(errors::middleware::ApiError::new()));
    log::info!("ready on: {}", url_port);
    srv.at("/").get(|_| async { Ok("oneroster ui\n") });
    srv.at("/auth/login").post(login);
    srv.at("/auth/check_token").get(check_token);
    srv.at("/orgs").get(get_all_orgs).put(put_orgs);
    // test
    srv.at("/academicSessions")
        .get(get_all_academic_sessions)
        .put(put_academic_sessions);
    srv.at("/periods").get(get_all_periods).put(put_periods);
    srv.at("/subjects").get(get_all_subjects).put(put_subjects);
    srv.at("/courses").get(get_all_courses).put(put_courses);
    srv.at("/classes").get(get_all_classes).put(put_classes);
    srv.at("/users").get(get_all_users).put(put_users);

    let mut authsrv = tide::with_state(srv.state().clone());
    authsrv.with(auth::middleware::Jwt::new(vec![
        "roster-core".to_string(),
        "roster".to_string(),
    ]));
    authsrv
        .at("/")
        .get(|_| async { Ok("hello protected world\n") });
    /*
    authsrv
        .at("/academicSessions")
        .get(get_all_academic_sessions);
    */
    authsrv.at("/academicSessions").put(put_academic_sessions);
    let mut adminsrv = tide::with_state(srv.state().clone());
    adminsrv.with(auth::middleware::Jwt::new(vec!["admin".to_string()]));
    adminsrv.at("/users").get(get_api_users);
    adminsrv.at("/user").post(create_api_user);
    adminsrv.at("/user/:uuid").delete(delete_api_user);

    srv.at("/admin").nest(adminsrv);
    srv.at("/ims/oneroster/v1p1").nest(authsrv);
    srv.listen(url_port).await?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creds {
    client_id: String,
    client_secret: String,
    scope: String,
}

async fn to_vec<T>(req: &mut Request<State>) -> Result<Vec<T>>
where
    for<'a> T: Deserialize<'a>,
{
    let s = req.body_string().await.unwrap();
    let v = serde_json::from_str(&s)?;
    Ok(v)
}

async fn login(mut req: tide::Request<State>) -> tide::Result {
    let creds: Creds = req.body_form().await?;
    log::info!("login attempt from: {}", creds.client_id);
    let token = auth::credentials::login(creds, &req.state().db).await?;
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
    let res = db::get_api_users("1".to_string(), "1".to_string(), &req.state().db).await?;
    Ok(tide::Response::builder(200).body(json!(res)).build())
}

async fn check_token(req: tide::Request<State>) -> tide::Result<String> {
    let token = auth::middleware::parse_auth_header(&req).await?;
    if auth::jwt::validate_token(token).await {
        return Ok("✔ Token valid\n".to_string());
    }
    Ok("✗ Token invalid\n".to_string())
}

// tests
#[cfg(test)]
#[async_std::test]
async fn db() -> Result<()> {
    let path = "sqlite:db/rust_test.db";
    db::init(path).await?;
    let pool = db::connect(path).await?;
    db::init_schema(&pool).await?;

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
