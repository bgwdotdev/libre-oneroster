use crate::server::{auth, Result, ServerError, State};
use futures::TryFutureExt;
use http_types::Method;

pub(crate) struct Jwt {
    scope: Vec<String>,
}

impl Jwt {
    pub(crate) fn new(scope: Vec<String>) -> Self {
        Self { scope }
    }
}

#[tide::utils::async_trait]
impl tide::Middleware<State> for Jwt {
    async fn handle(&self, req: tide::Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        let token = parse_auth_header(&req)
            .and_then(|t| async { auth::jwt::decode_token(t).await })
            .await?;
        parse_permission(&self.scope, req.method(), &token.claims.scope).await?;
        Ok(next.run(req).await)
    }
}

/// verifies the correct CRUD and ENDPOINT permissions are met in the scope string
async fn parse_permission(
    scopes: &Vec<String>,
    method: http_types::Method,
    target: &String,
) -> Result<()> {
    if let Some(method) = parse_method_permission(method).await {
        parse_scope_permission(scopes, method, &target).await?;
        log::debug!(
            "scope: {:?} does not meet requirements: {:?}, {:?}",
            target,
            scopes,
            method
        );
        return Ok(());
    }
    Err(ServerError::NoPermission)
}

/// converts endpoint methods into their relevant scope CRUD action
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

/// iterates a list of scopes to check if matching exists in scope string
async fn parse_scope_permission(scopes: &Vec<String>, method: &str, target: &String) -> Result<()> {
    for scope in scopes {
        let permission = scope.clone() + "." + method;
        if target.contains(&permission) {
            log::debug!("{:?} contains {:?}", target, &permission);
            return Ok(());
        }
    }
    Err(ServerError::NoPermission)
}

/// extracts the bearer token from the authorization header
pub(crate) async fn parse_auth_header(req: &tide::Request<State>) -> Result<String> {
    if let Some(bearer) = req.header("Authorization").and_then(|h| h.get(0)) {
        if let Some(token) = bearer.to_string().split(' ').nth(1).map(|t| t.to_string()) {
            return Ok(token.clone());
        }
    }
    Err(ServerError::NoBearerToken)
}
