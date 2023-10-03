use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthError {
    MissingClaim(String),
    InvalidFormat(String),
    TokenExpiredS(i64),
    JwtError(String),
}

impl From<jwt::Error> for AuthError {
    fn from(value: jwt::Error) -> Self {
        Self::JwtError(value.to_string())
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl std::error::Error for AuthError {}
