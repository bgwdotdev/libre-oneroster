mod auth;
mod db;
mod errors;

use crate::model;
use errors::*;
use tide::prelude::*;
use tide::utils::After;
use tide::Request;

type Result<T> = std::result::Result<T, ServerError>;

#[derive(Clone)]
pub(crate) struct State {
    db: sqlx::SqlitePool,
}

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

    srv.with(After(|mut r: tide::Response| async {
        if let Some(err) = r.downcast_error::<ServerError>() {
            println!("ERROR: {:?}", err);
            match err {
                ServerError::NoAuthorizedScopes
                | ServerError::NoBearerToken
                | ServerError::NoPermission
                | ServerError::InvalidLogin => {
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::Unauthorized,
                        description: Some(format!("{}", err)),
                        severity: Severity::Error,
                    };
                    r.set_status(403);
                    r.set_body(json!(ep));
                }
                ServerError::Jwt(_) => {
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::Unauthorized,
                        description: Some(format!("Invalid token")),
                        severity: Severity::Error,
                    };
                    r.set_status(403);
                    r.set_body(json!(ep));
                }
                ServerError::NoRecordDeleted => {
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::UnknownObject,
                        description: Some(format!("{}", err)),
                        severity: Severity::Error,
                    };
                    r.set_status(404);
                    r.set_body(json!(ep));
                }
                _ => println!("hi"),
            }
        };
        Ok(r)
    }));
    log::info!("ready on: {}", url_port);
    srv.at("/").get(|_| async { Ok("oneroster ui\n") });
    srv.at("/auth/login").post(login);
    srv.at("/auth/check_token").get(check_token);

    let mut authsrv = tide::with_state(srv.state().clone());
    authsrv.with(auth::middleware::Jwt::new(vec![
        "roster-core".to_string(),
        "roster".to_string(),
    ]));
    authsrv
        .at("/")
        .get(|_| async { Ok("hello protected world\n") });
    authsrv
        .at("/academicSessions")
        .get(get_all_academic_sessions);
    authsrv.at("/academicSessions").put(put_academic_sesions);
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
async fn put_academic_sesions(mut req: Request<State>) -> tide::Result {
    let j: Vec<model::AcademicSession> = req.body_json().await?;
    log::debug!("put req for: {:?}", j);
    db::put_academic_sessions(j, &req.state().db).await?;
    Ok(tide::Response::builder(200).build())
}

async fn get_all_academic_sessions(req: Request<State>) -> tide::Result {
    let json = db::get_all_academic_sessions(&req.state().db).await?;
    Ok(tide::Response::builder(200).body(json!(json)).build())
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
