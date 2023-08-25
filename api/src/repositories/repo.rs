use crate::{auth::error::AuthError, controllers::response::ApiResponse};
use rocket::serde::json::Json;
use rocket::{http, response::status};
use std::fmt::Display;

pub type ApiError<'a> = status::Custom<Json<ApiResponse<'a, String>>>;

#[derive(Debug)]
pub enum RepoError {
    AuthError(AuthError),
    DatabaseError(diesel::result::Error),
    DatabaseConnectionError(r2d2::Error),
    UserNameTaken(String),
    UserNotFound(String),
    InvalidCredentials,
    UserNotConfirmed(String),
    ValidationError(String),
}

impl RepoError {
    pub fn status(&self) -> http::Status {
        match self {
            RepoError::AuthError(e) => match e {
                AuthError::JwtError(_) => http::Status::InternalServerError,
                _ => http::Status::BadRequest,
            },
            RepoError::DatabaseError(_) => http::Status::InternalServerError,
            RepoError::DatabaseConnectionError(_) => http::Status::InternalServerError,
            RepoError::UserNameTaken(_) => http::Status::BadRequest,
            RepoError::UserNotFound(_) => http::Status::NotFound,
            RepoError::InvalidCredentials => http::Status::Unauthorized,
            RepoError::UserNotConfirmed(_) => http::Status::PaymentRequired,
            RepoError::ValidationError(_) => http::Status::BadRequest,
        }
    }
}

impl Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoError::AuthError(e) => e.fmt(f),
            RepoError::DatabaseError(e) => e.fmt(f),
            RepoError::DatabaseConnectionError(e) => e.fmt(f),
            RepoError::UserNameTaken(s) => {
                f.write_fmt(format_args!("Name {} is already in use.", s))
            }
            RepoError::UserNotFound(s) => {
                f.write_fmt(format_args!("User with {} was not found.", s))
            }
            RepoError::InvalidCredentials => f.write_str("Invalid credentials."),
            RepoError::UserNotConfirmed(s) => {
                f.write_fmt(format_args!("User {} is not confirmed", s))
            }
            RepoError::ValidationError(s) => f.write_str(s.as_str()),
        }
    }
}

impl From<RepoError> for status::Custom<Json<ApiResponse<'_, String>>> {
    fn from(value: RepoError) -> Self {
        status::Custom(value.status(), Json(ApiResponse::err(value.to_string())))
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
