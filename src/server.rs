mod auth;
mod db;

use crate::model;
use bcrypt;
use http_types::Method;
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
    db::init(path).await?;
    let pool = db::connect(path).await?;
    db::init_schema(&pool).await?;

    let state = State { db: pool };
    let url_port = "localhost:8080";
    let mut srv = tide::with_state(state);

    log::info!("ready on: {}", url_port);
    srv.at("/").get(|_| async { Ok("oneroster ui\n") });
    srv.at("/auth/login").post(login);
    srv.at("/auth/check_token").get(check_token);

    let mut authsrv = tide::with_state(srv.state().clone());
    authsrv.with(JwtMiddleware::new(vec!["roster-core".to_string()]));
    authsrv
        .at("/")
        .get(|_| async { Ok("hello protected world\n") });
    authsrv
        .at("/academicSessions")
        .get(get_all_academic_sessions);
    authsrv.at("/academicSessions").put(put_academic_sesions);
    let mut adminsrv = tide::with_state(authsrv.state().clone());
    adminsrv.with(JwtMiddleware::new(vec!["admin".to_string()]));
    adminsrv.at("/users").get(get_api_users);
    adminsrv.at("/user").post(create_api_user);
    adminsrv.at("/user/:uuid").delete(delete_api_user);

    authsrv.at("/admin").nest(adminsrv);
    srv.at("/ims/oneroster/v1p1").nest(authsrv);
    srv.listen(url_port).await?;
    Ok(())
}

// auth
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
    let compare = db::get_api_creds(&creds.client_id, &req.state().db).await?;
    let verify = bcrypt::verify(creds.client_secret, &compare.client_secret)?;
    if verify {
        // auth::verifyscope()?;
        let token = auth::create_token(creds.client_id, creds.scope).await?;
        return Ok(tide::Response::builder(200).body(json!(token)).build());
    }
    Ok(tide::Response::new(tide::StatusCode::Unauthorized))
}

async fn create_api_user(mut req: tide::Request<State>) -> tide::Result {
    let new: db::CreateApiUser = req.body_json().await?;
    let creds = db::create_api_user(new, &req.state().db).await?;
    Ok(tide::Response::builder(200).body(json!(creds)).build())
}

async fn delete_api_user(req: tide::Request<State>) -> tide::Result {
    let uuid = req.param("uuid")?;
    let res = db::delete_api_user(uuid, &req.state().db).await?;
    if res {
        return Ok(tide::Response::builder(200).build());
    }
    Ok(tide::Response::builder(404).build())
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
    if let Some(token) = parse_auth_header(&req).await {
        if auth::validate_token(token).await {
            return Ok("✔ Token valid\n".to_string());
        }
    }
    Ok("✗ Token invalid\n".to_string())
}

// jwt middleware
struct JwtMiddleware {
    scope: Vec<String>,
}

impl JwtMiddleware {
    fn new(scope: Vec<String>) -> Self {
        Self { scope }
    }
}

#[tide::utils::async_trait]
impl tide::Middleware<State> for JwtMiddleware {
    async fn handle(&self, req: tide::Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        if let Some(token) = parse_auth_header(&req).await {
            log::debug!("Authorization Header:\n{:?}", token);

            match auth::decode_token(token).await {
                Ok(t) => {
                    if parse_permission(&self.scope, req.method(), &t.claims.scope).await {
                        return Ok(next.run(req).await);
                    }
                }
                Err(_) => return Ok(tide::Response::builder(403).build()),
            }
        }
        Ok(tide::Response::builder(403).build())
    }
}

async fn parse_permission(
    scopes: &Vec<String>,
    method: http_types::Method,
    target: &String,
) -> bool {
    let method = parse_method_permission(method).await;
    if let Some(m) = method {
        if parse_scope_permission(scopes, m, &target).await {
            return true;
        }
        log::debug!(
            "scope: {:?} does not meet requirements: {:?}, {:?}",
            target,
            scopes,
            method
        );
    }
    false
}

async fn parse_method_permission<'a>(method: http_types::Method) -> Option<&'a str> {
    let result = match method {
        Method::Get => Some("readonly"),
        Method::Put => Some("createput"),
        Method::Delete => Some("delete"),
        Method::Post => Some("create"),
        _ => None,
    };
    result
}

async fn parse_scope_permission(scopes: &Vec<String>, method: &str, target: &String) -> bool {
    for scope in scopes {
        let permission = scope.clone() + "." + method;
        if target.contains(&permission) {
            log::debug!("{:?} contains {:?}", target, &permission);
            return true;
        }
    }
    false
}

async fn parse_auth_header(req: &tide::Request<State>) -> Option<String> {
    if let Some(bearer) = req.header("Authorization").and_then(|h| h.get(0)) {
        let token = bearer.to_string().split(' ').nth(1).map(|t| t.to_string());
        return token;
    }
    None
}

// tests
#[cfg(test)]
#[async_std::test]
async fn db() -> sqlx::Result<()> {
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
