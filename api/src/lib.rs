use crate::controllers::controller::ControllerRegisterer;
use crate::controllers::users::UsersController;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use repositories::repo::UserRepo;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    Build, Config, Request, Response, Rocket,
};
use std::env;

pub mod auth;
pub mod controllers;
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
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "*"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

pub fn build_rocket(secrets: &Secrets, user_repo: &'static dyn UserRepo) -> Rocket<Build> {
    rocket::build()
        .add(UsersController)
        .manage(secrets.clone())
        .manage(user_repo)
        .attach(CORS)
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
