use crate::controllers::controller::ControllerRegisterer;
use crate::controllers::users::UsersController;
use azure_storage_blobs::prelude::ClientBuilder;
use controllers::user_settings::UserSettingsController;
use controllers::{blob::BlobController, resources::ResourcesController};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use models::azure::AzureBlobSecrets;
use petompp_web_models::{error::Error, models::api_response::ApiResponse};
use repositories::{
    resources::repo::ResourcesRepo, user::repo::UserRepo, user_settings::repo::UserSettingsRepo,
};
use rocket::{catch, http::Status, serde::json::Json, Build, Rocket};
use rocket::{catchers, Request};
use std::env;

pub mod auth;
pub mod controllers;
pub mod models;
pub mod repositories;
pub mod schema;
pub mod services;

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

pub fn build_rocket(secrets: &Secrets, pg_pool: &'static PgPool) -> Rocket<Build> {
    let cors = rocket_cors::CorsOptions::default()
        .allow_credentials(true)
        .to_cors()
        .unwrap();

    rocket::build()
        .add(UsersController)
        .add(ResourcesController)
        .add(BlobController)
        .add(UserSettingsController)
        .mount("/", rocket_cors::catch_all_options_routes())
        .register("/", catchers![err])
        .attach(cors.clone())
        .manage(cors)
        .manage(secrets.clone())
        .manage(pg_pool)
        .manage::<&'static dyn UserRepo>(pg_pool)
        .manage::<&'static dyn ResourcesRepo>(pg_pool)
        .manage::<&'static dyn UserSettingsRepo>(pg_pool)
        .manage::<ClientBuilder>(AzureBlobSecrets::default().into())
}

pub fn get_connection_pool(secrets: &Secrets) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(secrets.database_url.clone());
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create pool")
}

#[catch(default)]
async fn err(status: Status, _req: &Request<'_>) -> Json<ApiResponse<'static, Error>> {
    Json(ApiResponse::err(Error::from(status)))
}
