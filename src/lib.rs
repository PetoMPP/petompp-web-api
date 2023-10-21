use crate::controllers::controller::ControllerRegisterer;
use crate::controllers::users::UsersController;
use controllers::resources::ResourcesController;
use controllers::{blog_meta::BlogMetaController, image::ImageController};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use petompp_web_models::services::filename::FilenameService;
use petompp_web_models::{error::Error, models::api_response::ApiResponse};
use repositories::{resources::repo::ResourcesRepo, user::repo::UserRepo};
use rocket::{catch, http::Status, serde::json::Json, Build, Rocket};
use rocket::{catchers, Request};
use services::azure_blob::{AzureBlobSecrets, AzureBlobService};
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
        .add(ImageController)
        .add(BlogMetaController)
        .mount("/", rocket_cors::catch_all_options_routes())
        .register("/", catchers![err])
        .attach(cors.clone())
        .manage(cors)
        .manage(secrets.clone())
        .manage(user_repo)
        .manage(resources_repo)
        .manage(AzureBlobService::new(AzureBlobSecrets::default()))
        .manage(FilenameService::default())
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
