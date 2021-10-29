use crate::server::Result;
use jsonwebtoken;
use std::time::SystemTime;
use tide::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Claims {
    aud: String,
    exp: u64,
    sub: String,
    pub(crate) scope: String,
}
// scopes:
// roster-core.readonly roster.readonly roster-demographics.readonly
// resource.readonly gradebook.readonly gradebook.createput gradebook.delete
// admin.readonly admin.create admin.delete

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TokenReturn {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
}

pub(crate) async fn create_token(
    id: String,
    scope: String,
    encode_key: &jsonwebtoken::EncodingKey,
) -> Result<TokenReturn> {
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let exp_in: u64 = 3600;
    let exp = SystemTime::now().duration_since(std::time::UNIX_EPOCH)?
        + std::time::Duration::from_secs(exp_in);
    let claims = Claims {
        aud: "localhost".to_string(),
        exp: exp.as_secs(),
        sub: id,
        scope: scope.clone(),
    };
    let token = jsonwebtoken::encode(&header, &claims, &encode_key)?;
    log::debug!("creating token:\n{}", &token);
    let result = TokenReturn {
        access_token: token,
        token_type: "bearer".to_string(),
        expires_in: exp_in,
        scope: scope.clone(),
    };
    Ok(result)
}

pub(crate) async fn decode_token(
    token: String,
    key: &jsonwebtoken::DecodingKey,
) -> Result<jsonwebtoken::TokenData<Claims>> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    let claims = jsonwebtoken::decode::<Claims>(&token, key, &validation)?;
    Ok(claims)
}

pub(crate) async fn validate_token(token: String, key: &jsonwebtoken::DecodingKey) -> bool {
    log::debug!("validating token:\n{}", token);
    match decode_token(token, key).await {
        Ok(t) => {
            log::debug!("validated:\n{:?}", t);
            true
        }
        Err(_) => false,
    }
}
