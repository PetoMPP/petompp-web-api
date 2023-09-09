use crate::models::password::PasswordRequirements;
use crate::{auth::error::AuthError, controllers::response::ApiResponse};
use rocket::serde::json::Json;
use rocket::{http, response::status};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub type ApiError<'a> = status::Custom<Json<ApiResponse<'a, Error>>>;

impl From<Error> for ApiError<'_> {
    fn from(value: Error) -> Self {
        status::Custom(value.status(), Json(ApiResponse::err(value)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    AuthError(AuthError),
    DatabaseError(String),
    DatabaseConnectionError(String),
    UserNameTaken(String),
    UserNotFound(String),
    InvalidCredentials,
    UserNotConfirmed(String),
    ValidationError(ValidationError),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ValidationError {
    Username(UsernameValidationError),
    Password(PasswordRequirements),
    Query(QueryValidationError),
    ResourceData(ResourceDataValidationError),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UsernameValidationError {
    InvalidLength(i32, i32),
    InvalidCharacters(Vec<char>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QueryValidationError {
    InvalidColumn(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResourceDataValidationError {
    KeyMismatch(String, String),
    KeyMissing,
    ValueMissing,
}

impl Error {
    pub fn status(&self) -> http::Status {
        match self {
            Error::AuthError(e) => match e {
                AuthError::JwtError(_) => http::Status::InternalServerError,
                _ => http::Status::BadRequest,
            },
            Error::DatabaseError(_) => http::Status::InternalServerError,
            Error::DatabaseConnectionError(_) => http::Status::InternalServerError,
            Error::UserNameTaken(_) => http::Status::BadRequest,
            Error::UserNotFound(_) => http::Status::NotFound,
            Error::InvalidCredentials => http::Status::Unauthorized,
            Error::UserNotConfirmed(_) => http::Status::PaymentRequired,
            Error::ValidationError(_) => http::Status::BadRequest,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        Self::DatabaseConnectionError(e.to_string())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::DatabaseError(e.to_string())
    }
}

impl From<AuthError> for Error {
    fn from(value: AuthError) -> Self {
        Error::AuthError(value)
    }
}

impl std::error::Error for Error {}
