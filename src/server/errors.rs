pub(crate) mod middleware;
use bcrypt;
use serde::{Deserialize, Serialize};
use std::{error, fmt};

#[derive(Debug)]
pub enum ServerError {
    Sqlx(sqlx::Error),
    Bcrypt(bcrypt::BcryptError),
    Time(std::time::SystemTimeError),
    Jwt(jsonwebtoken::errors::Error),
    Regex(regex::Error),
    Json(serde_json::Error),
    InvalidLogin,
    NoAuthorizedScopes,
    NoPermission,
    NoBearerToken,
    NoRecordDeleted,
    NoContent,
    InvalidFilterField,
    InvalidParameters,
    InvalidBlankSelectionField,
    NoDatabaseFound,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ServerError::Sqlx(ref e) => e.fmt(f),
            ServerError::Bcrypt(ref e) => e.fmt(f),
            ServerError::Time(ref e) => e.fmt(f),
            ServerError::Jwt(ref e) => e.fmt(f),
            ServerError::Regex(ref e) => e.fmt(f),
            ServerError::Json(ref e) => e.fmt(f),
            ServerError::InvalidLogin => write!(f, "Invalid username/password"),
            ServerError::NoAuthorizedScopes => write!(f, "No scopes were authorized for use"),
            ServerError::NoPermission => write!(f, "Incorrect scopes to access this resource"),
            ServerError::NoBearerToken => write!(f, "No bearer token found"),
            ServerError::NoRecordDeleted => write!(f, "No Record to delete"),
            ServerError::NoContent => write!(f, "No Content"),
            ServerError::InvalidFilterField => write!(f, "Invalid filter composition"),
            ServerError::InvalidParameters => write!(f, "Invalid parameter composition"),
            ServerError::InvalidBlankSelectionField => write!(f, "Invalid field composition"),
            ServerError::NoDatabaseFound => {
                write!(f, "No database found, check path or use --init to create")
            }
        }
    }
}

impl error::Error for ServerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ServerError::Sqlx(ref e) => Some(e),
            ServerError::Bcrypt(ref e) => Some(e),
            ServerError::Time(ref e) => Some(e),
            ServerError::Jwt(ref e) => Some(e),
            ServerError::Regex(ref e) => Some(e),
            ServerError::Json(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(err: sqlx::Error) -> ServerError {
        ServerError::Sqlx(err)
    }
}

impl From<bcrypt::BcryptError> for ServerError {
    fn from(err: bcrypt::BcryptError) -> ServerError {
        ServerError::Bcrypt(err)
    }
}

impl From<std::time::SystemTimeError> for ServerError {
    fn from(err: std::time::SystemTimeError) -> ServerError {
        ServerError::Time(err)
    }
}

impl From<jsonwebtoken::errors::Error> for ServerError {
    fn from(err: jsonwebtoken::errors::Error) -> ServerError {
        ServerError::Jwt(err)
    }
}

impl From<regex::Error> for ServerError {
    fn from(err: regex::Error) -> ServerError {
        ServerError::Regex(err)
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> ServerError {
        ServerError::Json(err)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CodeMajor {
    Success,
    Failure,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Severity {
    Status,
    Error,
    Warning,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CodeMinor {
    FullSuccess,
    UnknownObject,
    InvalidData,
    Unauthorized,
    InvalidSortField,
    InvalidFilterField,
    InvalidSelectionField,
    Forbidden,
    ServerBusy,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ErrorPayload {
    pub(crate) code_major: CodeMajor,
    pub(crate) severity: Severity,
    pub(crate) code_minor: CodeMinor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
}
