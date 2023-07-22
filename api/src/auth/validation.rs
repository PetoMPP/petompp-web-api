use super::error::AuthError;
use crate::{controllers::users::User, Secrets};
use hmac::{digest::KeyInit, Hmac};
use jwt::{SignWithKey, VerifyWithKey};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u32,
    pub exp: u64,
    pub acs: u32,
}

const SUB_CLAIM: &str = "sub";
const EXP_CLAIM: &str = "exp";
const ACS_CLAIM: &str = "acs";

impl Into<BTreeMap<String, String>> for Claims {
    fn into(self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        map.insert(SUB_CLAIM.to_string(), self.sub.to_string());
        map.insert(EXP_CLAIM.to_string(), self.exp.to_string());
        map.insert(ACS_CLAIM.to_string(), self.acs.to_string());
        map
    }
}

impl TryFrom<BTreeMap<String, String>> for Claims {
    type Error = AuthError;

    fn try_from(value: BTreeMap<String, String>) -> Result<Self, Self::Error> {
        let sub = get_claim_value(&value, SUB_CLAIM)?;
        let exp = get_claim_value(&value, EXP_CLAIM)?;
        if exp < chrono::Utc::now().timestamp() as u64 {
            return Err(AuthError::TokenExpired(chrono::Duration::seconds(
                exp as i64 - chrono::Utc::now().timestamp(),
            )));
        }
        let acs = get_claim_value(&value, ACS_CLAIM)?;
        Ok(Self { sub, exp, acs })
    }
}

fn get_claim_value<T: FromStr>(
    claims: &BTreeMap<String, String>,
    claim: &'static str,
) -> Result<T, AuthError> {
    claims
        .get(claim)
        .ok_or(AuthError::MissingClaim(claim))?
        .parse::<T>()
        .map_err(|_| AuthError::InvalidFormat(claim))
}

impl Claims {
    pub fn new_from_user(user: &User) -> Self {
        Self {
            sub: user.id,
            exp: chrono::Utc::now().timestamp() as u64 + 60 * 60,
            acs: 0,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, ()> {
        let secrets = request.rocket().state::<Secrets>().unwrap();
        let Some(token) = request.headers().get_one("Authorization") else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        let Some(token) = token.strip_prefix("Bearer ") else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        let Ok(claims) = validate_token(secrets, token) else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        Outcome::Success(claims)
    }
}

pub fn create_token(secrets: &Secrets, user: &User) -> Result<String, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secrets.api_secret.as_bytes()).unwrap();
    let claims: BTreeMap<String, String> = Claims::new_from_user(user).into();
    Ok(claims.sign_with_key(&key)?)
}

pub fn validate_token(secrets: &Secrets, token: &str) -> Result<Claims, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secrets.api_secret.as_bytes()).unwrap();
    let token_data: BTreeMap<String, String> = token.verify_with_key(&key)?;

    Claims::try_from(token_data)
}
