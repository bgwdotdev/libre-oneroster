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

// TODO: get keys from user
lazy_static::lazy_static! {
    static ref JWT_ENCODE_KEY: jsonwebtoken::EncodingKey = {
        jsonwebtoken::EncodingKey::from_rsa_pem(include_bytes!("../../../certs/localhost.key.pem"))
            .expect("Problem loading private key")
    };
    // jwt crate doesn't support x509 so must extract pub key with openssl, see:
    // https://github.com/Keats/jsonwebtoken/issues/127
    static ref JWT_DECODE_KEY: jsonwebtoken::DecodingKey = {
        let cert = openssl::x509::X509::from_pem(include_bytes!("../../../certs/localhost.pem"))
            .expect("problem loading pub pem");
        let pem = cert.public_key().unwrap().rsa().unwrap().public_key_to_pem().unwrap();
        let pubkey = jsonwebtoken::DecodingKey::from_rsa_pem(&pem).unwrap();
        return pubkey;
    };
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct TokenReturn {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
}

pub(crate) async fn create_token(id: String, scope: String) -> Result<TokenReturn> {
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
    let token = jsonwebtoken::encode(&header, &claims, &JWT_ENCODE_KEY)?;
    log::debug!("creating token:\n{}", &token);
    let result = TokenReturn {
        access_token: token,
        token_type: "bearer".to_string(),
        expires_in: exp_in,
        scope: scope.clone(),
    };
    Ok(result)
}

pub(crate) async fn decode_token(token: String) -> Result<jsonwebtoken::TokenData<Claims>> {
    let val = jsonwebtoken::Validation {
        algorithms: vec![jsonwebtoken::Algorithm::RS256],
        ..Default::default()
    };
    let claims = jsonwebtoken::decode::<Claims>(&token, &JWT_DECODE_KEY, &val)?;
    Ok(claims)
}

pub(crate) async fn validate_token(token: String) -> bool {
    log::debug!("validating token:\n{}", token);
    match decode_token(token).await {
        Ok(t) => {
            log::debug!("validated:\n{:?}", t);
            true
        }
        Err(_) => false,
    }
}
