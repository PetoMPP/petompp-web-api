use rocket::{http, response::status};
use std::fmt::Display;

use crate::{
    auth::error::AuthError,
    models::{credentials::Credentials, user::User},
    Secrets,
};

#[derive(Debug)]
pub enum RepoError {
    AuthError(AuthError),
    DatabaseError(diesel::result::Error),
    DatabaseConnectionError(r2d2::Error),
    UserAlreadyExists(String),
    UserNotFound(String),
    InvalidCredentials,
    UserNotConfirmed(String),
}

impl Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoError::AuthError(e) => e.fmt(f),
            RepoError::DatabaseError(e) => e.fmt(f),
            RepoError::DatabaseConnectionError(e) => e.fmt(f),
            RepoError::UserAlreadyExists(s) => f.write_fmt(format_args!("{:?}, {}", self, s)),
            RepoError::UserNotFound(s) => f.write_fmt(format_args!("{:?}, {}", self, s)),
            RepoError::InvalidCredentials => f.write_fmt(format_args!("{:?}", self)),
            RepoError::UserNotConfirmed(s) => f.write_fmt(format_args!("{:?}, {}", self, s)),
        }
    }
}

impl Into<status::Custom<String>> for RepoError {
    fn into(self) -> status::Custom<String> {
        match self {
            RepoError::AuthError(e) => match e {
                AuthError::JwtError(e) => {
                    status::Custom(http::Status::InternalServerError, e.to_string())
                }
                _ => status::Custom(http::Status::BadRequest, e.to_string()),
            },
            RepoError::DatabaseError(e) => {
                status::Custom(http::Status::InternalServerError, e.to_string())
            }
            RepoError::DatabaseConnectionError(e) => {
                status::Custom(http::Status::InternalServerError, e.to_string())
            }
            RepoError::UserAlreadyExists(e) => {
                status::Custom(http::Status::InternalServerError, e.to_string())
            }
            RepoError::UserNotFound(e) => status::Custom(http::Status::NotFound, e.to_string()),
            RepoError::InvalidCredentials => {
                status::Custom(http::Status::Unauthorized, self.to_string())
            }
            RepoError::UserNotConfirmed(e) => {
                status::Custom(http::Status::PaymentRequired, e.to_string())
            }
        }
    }
}

impl From<r2d2::Error> for RepoError {
    fn from(e: r2d2::Error) -> Self {
        Self::DatabaseConnectionError(e)
    }
}

impl From<diesel::result::Error> for RepoError {
    fn from(e: diesel::result::Error) -> Self {
        Self::DatabaseError(e)
    }
}

impl From<AuthError> for RepoError {
    fn from(value: AuthError) -> Self {
        RepoError::AuthError(value)
    }
}

impl std::error::Error for RepoError {}

pub trait UserRepo {
    fn create(&self, credentials: Credentials) -> Result<User, RepoError>;
    fn login(&self, credentials: Credentials, secrets: &Secrets) -> Result<String, RepoError>;
    fn get_self(&self, user_id: i32) -> Result<User, RepoError>;
    fn activate(&self, user_id: i32) -> Result<User, RepoError>;
}
