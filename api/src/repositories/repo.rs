use crate::{
    auth::error::AuthError,
    models::{credentials::Credentials, user::User},
    Secrets,
};
use rocket::{async_trait, http::Status, outcome::Outcome, request::FromRequest, Request};
use rocket::{http, response::status};
use std::fmt::Display;

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
            RepoError::UserAlreadyExists(s) => {
                f.write_fmt(format_args!("User ({}) already exists.", s))
            }
            RepoError::UserNotFound(s) => f.write_fmt(format_args!("User ({}) was not found.", s)),
            RepoError::InvalidCredentials => f.write_str("Invalid credentials."),
            RepoError::UserNotConfirmed(s) => {
                f.write_fmt(format_args!("User ({}) is not confirmed", s))
            }
        }
    }
}

impl From<RepoError> for status::Custom<String> {
    fn from(val: RepoError) -> Self {
        match &val {
            RepoError::AuthError(e) => match e {
                AuthError::JwtError(_) => {
                    status::Custom(http::Status::InternalServerError, val.to_string())
                }
                _ => status::Custom(http::Status::BadRequest, val.to_string()),
            },
            RepoError::DatabaseError(_) => {
                status::Custom(http::Status::InternalServerError, val.to_string())
            }
            RepoError::DatabaseConnectionError(_) => {
                status::Custom(http::Status::InternalServerError, val.to_string())
            }
            RepoError::UserAlreadyExists(_) => {
                status::Custom(http::Status::BadRequest, val.to_string())
            }
            RepoError::UserNotFound(_) => status::Custom(http::Status::NotFound, val.to_string()),
            RepoError::InvalidCredentials => {
                status::Custom(http::Status::Unauthorized, val.to_string())
            }
            RepoError::UserNotConfirmed(_) => {
                status::Custom(http::Status::PaymentRequired, val.to_string())
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

pub trait UserRepo: Send + Sync {
    fn create(&self, credentials: Credentials) -> Result<User, RepoError>;
    fn login(&self, credentials: Credentials, secrets: &Secrets) -> Result<String, RepoError>;
    fn get_self(&self, user_id: i32) -> Result<User, RepoError>;
    fn activate(&self, user_id: i32) -> Result<User, RepoError>;
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r dyn UserRepo {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        let pool = request
            .guard::<&rocket::State<&dyn UserRepo>>()
            .await
            .unwrap();
        Outcome::Success(*pool.inner())
    }
}
