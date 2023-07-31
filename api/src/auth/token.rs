use super::{claims::Claims, error::AuthError};
use crate::{models::user::User, Secrets};
use hmac::{digest::KeyInit, Hmac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

pub fn create_token(secrets: &Secrets, user: &User) -> Result<String, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secrets.api_secret.as_bytes()).unwrap();
    let claims: BTreeMap<String, String> = Claims::try_from(user.clone())?.into();
    Ok(claims.sign_with_key(&key)?)
}

pub fn validate_token(secrets: &Secrets, token: &str) -> Result<Claims, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secrets.api_secret.as_bytes()).unwrap();
    let token_data: BTreeMap<String, String> = token.verify_with_key(&key)?;

    Claims::try_from(token_data)
}
