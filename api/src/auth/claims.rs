use super::{error::AuthError, token::validate_token};
use crate::{
    models::{role::Role, user::User},
    Secrets,
};
use rocket::{http::Status, outcome::Outcome, request::FromRequest, Request};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Clone)]
pub struct Claims {
    pub sub: i32,
    pub exp: u64,
    pub acs: Role,
}

const SUB_CLAIM: &str = "sub";
const EXP_CLAIM: &str = "exp";
const ACS_CLAIM: &str = "acs";

impl From<Claims> for BTreeMap<String, String> {
    fn from(val: Claims) -> Self {
        let mut map = BTreeMap::new();
        map.insert(SUB_CLAIM.to_string(), val.sub.to_string());
        map.insert(EXP_CLAIM.to_string(), val.exp.to_string());
        map.insert(ACS_CLAIM.to_string(), val.acs.to_string());
        map
    }
}

impl TryFrom<BTreeMap<String, String>> for Claims {
    type Error = AuthError;

    fn try_from(value: BTreeMap<String, String>) -> Result<Self, Self::Error> {
        let sub = get_claim_value(&value, SUB_CLAIM)?;
        let exp = get_claim_value(&value, EXP_CLAIM)?;
        match (exp as i64) - chrono::Utc::now().timestamp() {
            ref x if x < &0 => Err(AuthError::TokenExpiredS(-*x)),
            _ => Ok(Self {
                sub,
                exp,
                acs: get_claim_value(&value, ACS_CLAIM)?,
            }),
        }
    }
}

fn get_claim_value<'a, T: FromStr>(
    claims: &BTreeMap<String, String>,
    claim: &'a str,
) -> Result<T, AuthError> {
    claims
        .get(claim)
        .ok_or(AuthError::MissingClaim(claim.to_string()))?
        .parse::<T>()
        .map_err(|_| AuthError::InvalidFormat(claim.to_string()))
}

impl TryFrom<User> for Claims {
    type Error = AuthError;

    fn try_from(value: User) -> Result<Self, Self::Error> {
        match value.id {
            Some(id) => Ok(Self {
                sub: id,
                exp: chrono::Utc::now().timestamp() as u64 + 60 * 60,
                acs: value.role,
            }),
            None => Err(AuthError::InvalidFormat("User id".to_string())),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
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

pub struct AdminClaims(Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminClaims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        let Outcome::Success(claims) = request.guard::<Claims>().await else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        if claims.acs != Role::Admin {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        Outcome::Success(Self(claims))
    }
}
