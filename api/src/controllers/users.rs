use crate::{
    auth::claims::{AdminClaims, Claims},
    controllers::controller::Controller,
    models::credentials::Credentials,
    repositories::repo::UserRepo,
    PgPool,
};
use rocket::{get, http::Status, post, response::status, routes, serde::json::Json, Build, State};

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

#[post("/", data = "<credentials>")]
async fn create(credentials: Json<Credentials>, pool: &State<PgPool>) -> status::Custom<String> {
    match pool.create(credentials.into_inner()) {
        Ok(user) => status::Custom(Status::Ok, serde_json::to_string_pretty(&user).unwrap()),
        Err(e) => e.into(),
    }
}

#[post("/login", data = "<credentials>")]
async fn login(
    credentials: Json<Credentials>,
    pool: &State<PgPool>,
    secrets: &State<crate::Secrets>,
) -> status::Custom<String> {
    match pool.login(credentials.into_inner(), secrets) {
        Ok(token) => status::Custom(Status::Ok, token),
        Err(e) => e.into(),
    }
}

#[get("/")]
async fn get_self(claims: Claims, pool: &State<PgPool>) -> status::Custom<String> {
    match pool.get_self(claims.sub) {
        Ok(user) => status::Custom(Status::Ok, serde_json::to_string_pretty(&user).unwrap()),
        Err(e) => e.into(),
    }
}

#[post("/<id>/activate")]
async fn activate(_claims: AdminClaims, id: i32, pool: &State<PgPool>) -> status::Custom<String> {
    match pool.activate(id) {
        Ok(_) => status::Custom(Status::Ok, "User activated!".to_string()),
        Err(e) => e.into(),
    }
}
