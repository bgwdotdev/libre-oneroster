use crate::server;
use bcrypt;
use rand::{rngs, Rng, RngCore};
use uuid::Uuid;

pub(crate) struct NewCreds {
    pub(crate) creds: server::Creds,
    pub(crate) encrypt: String,
}

pub(crate) async fn verify_scopes(current: &String, requested: &String) -> Option<String> {
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
        return Some(m);
    }
    None
}

pub(crate) async fn generate_credentials() -> Result<NewCreds, bcrypt::BcryptError> {
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
pub(crate) async fn generate_password() -> Result<(String, String), bcrypt::BcryptError> {
    let length = rngs::OsRng.gen_range(32..40);
    let mut key = vec![0u8; length];
    rngs::OsRng.fill_bytes(&mut key);

    let plaintext = hex::encode(&key);
    let encrypt = bcrypt::hash(&plaintext, 12)?;
    Ok((plaintext, encrypt))
}
