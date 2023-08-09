use super::response::ApiResponse;
use crate::{
    auth::{
        claims::{AdminClaims, Claims},
        error::AuthError,
        token::create_token,
    },
    controllers::controller::Controller,
    models::{credentials::Credentials, role::Role, user::User, user_name::UserName},
    repositories::repo::{RepoError, UserRepo},
};
use rocket::{get, post, response::status, routes, serde::json::Json, Build, State};
use serde::{Deserialize, Serialize};

pub struct UsersController;

impl Controller for UsersController {
    fn path(&self) -> &'static str {
        "/users"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![create, login, get_self, activate]
    }

    fn add_managed(&self, rocket: rocket::Rocket<Build>) -> rocket::Rocket<Build> {
        rocket
    }
}

type ApiError<'a> = status::Custom<Json<ApiResponse<'a, String>>>;

#[post("/", data = "<credentials>")]
async fn create(
    credentials: Json<Credentials>,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<User>>, ApiError> {
    let user = User::new(
        credentials.name.clone(),
        credentials.password.clone(),
        Role::User,
    )?;
    let user = pool.create(&user)?;
    Ok(Json(ApiResponse::ok(user)))
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    user: User,
}

#[post("/login", data = "<credentials>")]
async fn login<'a>(
    credentials: Json<Credentials>,
    pool: &'a dyn UserRepo,
    secrets: &State<crate::Secrets>,
) -> Result<Json<ApiResponse<'a, LoginResponse>>, ApiError<'a>> {
    let user = pool.get_by_name(credentials.name.to_ascii_lowercase())?;
    if !user.password.verify(credentials.password.clone()) {
        return Err(RepoError::InvalidCredentials.into());
    }
    if !user.confirmed {
        return Err(RepoError::UserNotConfirmed(credentials.name.to_string()).into());
    }
    let token = create_token(secrets, &user).map_err(<AuthError as Into<RepoError>>::into)?;
    Ok(Json(ApiResponse::ok(LoginResponse {token, user})))
}

#[get("/")]
async fn get_self(
    claims: Claims,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<User>>, ApiError> {
    let user = pool.get_by_id(claims.sub)?;
    Ok(Json(ApiResponse::ok(user)))
}

#[post("/<id>/activate")]
async fn activate(
    _claims: AdminClaims,
    id: i32,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<User>>, ApiError> {
    let user = pool.activate(id)?;
    Ok(Json(ApiResponse::ok(user)))
}
