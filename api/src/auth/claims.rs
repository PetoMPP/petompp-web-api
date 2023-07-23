use super::{error::AuthError, token::validate_token};
use crate::{
    models::user::{Role, User},
    Secrets,
};
use rocket::{http::Status, outcome::Outcome, request::FromRequest, Request};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Clone)]
pub struct Claims {
    pub sub: u32,
    pub exp: u64,
    pub acs: Role,
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
            acs: user.role,
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
