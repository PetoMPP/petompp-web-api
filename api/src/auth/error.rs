use std::fmt::Display;

use chrono::Duration;

#[derive(Debug)]
pub enum AuthError {
    MissingClaim(&'static str),
    InvalidFormat(&'static str),
    TokenExpired(Duration),
    JwtError(jwt::Error),
}

impl From<jwt::Error> for AuthError {
    fn from(value: jwt::Error) -> Self {
        Self::JwtError(value)
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            AuthError::MissingClaim(c) => f.write_fmt(format_args!("Claim {} is missing", c)),
            AuthError::InvalidFormat(c) => {
                f.write_fmt(format_args!("Claim {} has invalid format", c))
            }
            AuthError::TokenExpired(d) => f.write_fmt(format_args!(
                "Token is expired by {} seconds",
                d.num_seconds()
            )),
            AuthError::JwtError(jwt) => jwt.fmt(f),
        };
    }
}

impl std::error::Error for AuthError {}
