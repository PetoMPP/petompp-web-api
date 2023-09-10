use crate::controllers::users::UsersController;
use crate::controllers::{controller::ControllerRegisterer, response::ApiResponse};
use controllers::resources::ResourcesController;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use error::Error;
use repositories::{resources::repo::ResourcesRepo, user::repo::UserRepo};
use rocket::{catch, http::Status, serde::json::Json, Build, Config, Rocket};
use rocket::{catchers, Request};
use std::env;

pub mod auth;
pub mod controllers;
pub mod error;
pub mod models;
pub mod repositories;
pub mod schema;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone, Debug)]
pub struct Secrets {
    pub api_secret: String,
    pub database_url: String,
}

impl Default for Secrets {
    fn default() -> Self {
        let api_secret = env::var("API_SECRET").expect("API_SECRET must be set");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        Self {
            api_secret,
            database_url,
        }
    }
}

pub fn build_rocket(
    secrets: &Secrets,
    user_repo: &'static dyn UserRepo,
    resources_repo: &'static dyn ResourcesRepo,
) -> Rocket<Build> {
    let cors = rocket_cors::CorsOptions::default()
        .allow_credentials(true)
        .to_cors()
        .unwrap();

    rocket::build()
        .add(UsersController)
        .add(ResourcesController)
        .mount("/", rocket_cors::catch_all_options_routes())
        .register("/", catchers![err])
        .attach(cors.clone())
        .manage(cors)
        .manage(secrets.clone())
        .manage(user_repo)
        .manage(resources_repo)
        .configure(Config {
            port: 16969,
            address: "0.0.0.0".parse().unwrap(),
            ..Default::default()
        })
}

pub fn get_connection_pool(secrets: &Secrets) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(secrets.database_url.clone());
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create pool")
}

#[catch(default)]
fn err(status: Status, _req: &Request) -> Json<ApiResponse<'static, Error>> {
    Json(ApiResponse::err(Error::Status(
        status.code,
        status.to_string(),
    )))
}
