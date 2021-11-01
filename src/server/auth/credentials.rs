use crate::server;
use crate::server::{auth::jwt, db, Result};
use bcrypt;
use rand::{rngs, Rng, RngCore};
use uuid::Uuid;

pub(crate) struct NewCreds {
    pub(crate) creds: server::Creds,
    pub(crate) encrypt: String,
}

// TODO: write tests in rust
pub(crate) async fn login(
    creds: server::Creds,
    db: &sqlx::SqlitePool,
    key: &jsonwebtoken::EncodingKey,
) -> Result<jwt::TokenReturn> {
    let compare = db::get_api_creds(&creds.client_id, db).await;
    match compare {
        Ok(compare) => {
            let verify = bcrypt::verify(&creds.client_secret, &compare.client_secret)?;
            if verify {
                let scopes = verify_scopes(&compare.scope, &creds.scope).await?;
                let token = jwt::create_token(creds.client_id, scopes, key).await?;
                return Ok(token);
            }
        }
        Err(_) => {
            // Security
            // This case is designed to negate a timing attack by always hashing/evaluating the
            // user password even if the username is incorrect to prevent username bruteforcing via
            // time delta comparisons between hashing/non-hashing error operations
            bcrypt::verify(
                &creds.client_secret,
                "$2b$12$54Zvtx.e/V/nRPo0PUYrxOHqXZywSKzM7LLFqC/p59F0x87SsZdvW",
            )?;
            // This case should ALWAYS FAIL even if the password matches the above static hash
            return Err(server::ServerError::InvalidLogin);
        }
    };
    Err(server::ServerError::InvalidLogin)
}

pub(crate) async fn verify_scopes(current: &String, requested: &String) -> Result<String> {
    let mut matches: Vec<&str> = vec![];
    log::debug!("{}, {}", current, requested);
    for r in requested.split(' ') {
        for c in current.split(' ') {
            if r.eq(c) {
                matches.push(c);
            }
        }
    }
    log::debug!("allowed scopes: {:?}", matches);
    if matches.len() >= 1 {
        let m = matches.join(" ");
        return Ok(m);
    }
    Err(server::ServerError::NoAuthorizedScopes)
}

pub(crate) async fn generate_credentials() -> Result<NewCreds> {
    let (client_secret, encrypt) = generate_password().await?;
    let scope = "changeme".to_string();
    let creds = NewCreds {
        creds: server::Creds {
            client_id: Uuid::new_v4().to_hyphenated().to_string(),
            client_secret,
            scope,
        },
        encrypt,
    };
    Ok(creds)
}

/// Creates a variable length hex password using cryptographically
/// secure number generators backed by the OS
pub(crate) async fn generate_password() -> Result<(String, String)> {
    let length = rngs::OsRng.gen_range(32..40);
    let mut key = vec![0u8; length];
    rngs::OsRng.fill_bytes(&mut key);

    let plaintext = hex::encode(&key);
    let encrypt = bcrypt::hash(&plaintext, 12)?;
    Ok((plaintext, encrypt))
}
