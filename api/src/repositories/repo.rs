use super::query_config::QueryConfig;
use crate::{auth::error::AuthError, controllers::response::ApiResponse, models::user::User};
use rocket::{
    async_trait, http::Status, outcome::Outcome, request::FromRequest, serde::json::Json, Request,
};
use rocket::{http, response::status};
use std::fmt::Display;

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

pub trait UserRepo: Send + Sync {
    fn create(&self, user: &User) -> Result<User, RepoError>;
    fn get_by_name(&self, normalized_name: String) -> Result<User, RepoError>;
    fn get_by_id(&self, id: i32) -> Result<User, RepoError>;
    fn get_all(&self, query_config: &QueryConfig) -> Result<Vec<Vec<User>>, RepoError>;
    fn activate(&self, id: i32) -> Result<User, RepoError>;
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
