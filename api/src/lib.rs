use crate::controllers::controller::ControllerRegisterer;
use crate::controllers::users::UsersController;
use crate::extensions::extension::Extension;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use rocket::{Build, Config, Rocket};
use std::env;

pub mod auth;
pub mod controllers;
pub mod extensions;
pub mod models;
pub mod schema;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub struct Secrets {
    pub api_secret: String,
}

impl Default for Secrets {
    fn default() -> Self {
        let api_secret = env::var("API_SECRET").unwrap_or("shhhdonttellanyoneaboutit".to_string());
        Self { api_secret }
    }
}

pub fn build_rocket() -> Rocket<Build> {
    Extension(rocket::build())
        .add(UsersController)
        .into()
        .manage(Secrets::default())
        .manage(get_connection_pool())
        .configure(Config {
            port: 16969,
            address: "0.0.0.0".parse().unwrap(),
            ..Default::default()
        })
}

fn get_connection_pool() -> PgPool {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create pool")
}
