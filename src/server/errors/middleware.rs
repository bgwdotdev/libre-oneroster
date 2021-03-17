use super::*;
use crate::server;
use serde_json::json;

pub(crate) struct ApiError {}

impl ApiError {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[tide::utils::async_trait]
impl tide::Middleware<server::State> for tide::utils::After<ApiError> {
    async fn handle(
        &self,
        request: tide::Request<server::State>,
        next: tide::Next<'_, server::State>,
    ) -> tide::Result<tide::Response> {
        //log::trace!("{:?}", r);
        let mut r = next.run(request).await;
        if let Some(err) = r.downcast_error::<ServerError>() {
            log::warn!("API request error: {:?}", err);
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
                ServerError::Sqlx(ref e) => {
                    log::error!("API sql error: {}", e);
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::Forbidden, // None?
                        description: None,
                        severity: Severity::Error,
                    };
                    r.set_status(500);
                    r.set_body(json!(ep));
                }
                ServerError::Bcrypt(ref e) => {
                    log::error!("API bcrypt error: {}", e);
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::Forbidden, // None?
                        description: None,
                        severity: Severity::Error,
                    };
                    r.set_status(500);
                    r.set_body(json!(ep));
                }
                ServerError::Time(ref e) => {
                    log::error!("API time error: {}", e);
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::Forbidden, // None?
                        description: None,
                        severity: Severity::Error,
                    };
                    r.set_status(500);
                    r.set_body(json!(ep));
                }
                ServerError::Regex(ref e) => {
                    log::error!("API regex error: {}", e);
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::Forbidden, // None?
                        description: None,
                        severity: Severity::Error,
                    };
                    r.set_status(500);
                    r.set_body(json!(ep));
                }
                ServerError::InvalidFilterField
                | ServerError::InvalidParameters
                | ServerError::InvalidBlankSelectionField => {
                    let ep = ErrorPayload {
                        code_major: CodeMajor::Failure,
                        code_minor: CodeMinor::InvalidData,
                        description: Some(format!("{}", err)),
                        severity: Severity::Error,
                    };
                    r.set_status(400);
                    r.set_body(json!(ep));
                }
            }
        };
        Ok(r)
    }
}
