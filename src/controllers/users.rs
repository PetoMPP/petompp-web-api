use crate::{
    auth::{
        claims::{AdminClaims, Claims},
        token::create_token,
    },
    controllers::controller::Controller,
    models::{role::Role, user::User},
    repositories::{query_config::QueryConfig, user::repo::UserRepo},
};
use petompp_web_models::{
    error::{ApiError, AuthError, Error, RegisterError, UserError},
    models::{
        api_response::ApiResponse, credentials::Credentials,
        password_requirements::PasswordRequirements, requirement::Requirements, user::UserData,
        username_requirements::UsernameRequirements,
    },
};
use rocket::{delete, get, post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};

pub struct UsersController;

impl Controller for UsersController {
    fn path(&self) -> &'static str {
        "/users"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![create, login, get_self, activate, get_all, delete]
    }
}

#[post("/", data = "<credentials>")]
fn create(
    credentials: Json<Credentials>,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<UserData>>, ApiError> {
    let username_errors = match UsernameRequirements::default().validate(&credentials.name.as_str())
    {
        Ok(_) => Vec::new(),
        Err(e) => e.into_iter().map(|e| e.to_string()).collect::<Vec<_>>(),
    };
    let password_errors =
        match PasswordRequirements::default().validate(&credentials.password.as_str()) {
            Ok(_) => Vec::new(),
            Err(e) => e.into_iter().map(|e| e.to_string()).collect::<Vec<_>>(),
        };
    if !username_errors.is_empty() || !password_errors.is_empty() {
        return Err(Error::Register(RegisterError {
            username_errors,
            password_errors,
        })
        .into());
    }
    let user = User::new(
        credentials.name.clone(),
        credentials.password.clone(),
        Role::User,
    );
    let user = pool.create(&user)?;
    Ok(Json(ApiResponse::ok(user.into())))
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    user: UserData,
}

#[post("/login", data = "<credentials>")]
async fn login<'a>(
    credentials: Json<Credentials>,
    pool: &'a dyn UserRepo,
    secrets: &State<crate::Secrets>,
) -> Result<Json<ApiResponse<'a, LoginResponse>>, ApiError<'a>> {
    let user = match pool.get_by_name(credentials.name.to_ascii_lowercase())? {
        Some(user) if user.password.verify(credentials.password.clone()) => user,
        _ => return Err(Error::User(UserError::InvalidCredentials).into()),
    };
    if !user.confirmed {
        return Err(Error::User(UserError::NotConfirmed(credentials.name.to_string())).into());
    }
    let token = create_token(secrets, &user).map_err(<AuthError as Into<Error>>::into)?;
    Ok(Json(ApiResponse::ok(LoginResponse {
        token,
        user: user.into(),
    })))
}

#[get("/")]
async fn get_self(
    claims: Claims,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<UserData>>, ApiError> {
    let user = pool
        .get_by_id(claims.sub)?
        .ok_or_else(|| Error::User(UserError::NotFound(claims.sub.to_string())))?;
    Ok(Json(ApiResponse::ok(user.into())))
}

#[get("/all?<query..>")]
fn get_all(
    _claims: AdminClaims,
    query: QueryConfig,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<Vec<Vec<UserData>>>>, ApiError> {
    let users = pool
        .get_all(&query)?
        .into_iter()
        .map(|us| us.into_iter().map(|u| u.into()).collect::<Vec<_>>())
        .collect::<Vec<Vec<_>>>();
    Ok(Json(ApiResponse::ok(users)))
}

#[post("/<id>/activate")]
async fn activate(
    _claims: AdminClaims,
    id: i32,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<UserData>>, ApiError> {
    let user = pool
        .activate(id)?
        .ok_or_else(|| Error::User(UserError::NotFound(id.to_string())))?;
    Ok(Json(ApiResponse::ok(user.into())))
}

#[delete("/<id>")]
async fn delete(
    _claims: AdminClaims,
    id: i32,
    pool: &dyn UserRepo,
) -> Result<Json<ApiResponse<UserData>>, ApiError> {
    let user = pool
        .delete(id)?
        .ok_or_else(|| Error::User(UserError::NotFound(id.to_string())))?;
    Ok(Json(ApiResponse::ok(user.into())))
}
