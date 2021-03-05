use crate::server::auth;
use crate::server::State;
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
        if let Some(token) = parse_auth_header(&req).await {
            log::debug!("Authorization Header:\n{:?}", token);

            match auth::jwt::decode_token(token).await {
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

/// verifies the correct CRUD and ENDPOINT permissions are met in the scope string
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

/// extracts the bearer token from the authorization header
pub(crate) async fn parse_auth_header(req: &tide::Request<State>) -> Option<String> {
    if let Some(bearer) = req.header("Authorization").and_then(|h| h.get(0)) {
        let token = bearer.to_string().split(' ').nth(1).map(|t| t.to_string());
        return token.clone();
    }
    None
}
