use std::env;
use crate::controllers::users::UsersController;
use crate::extensions::extension::Extension;
use crate::controllers::controller::ControllerRegisterer;
use rocket::{Config, Rocket, Build};

pub mod auth;
pub mod controllers;
pub mod extensions;
pub mod models;

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
        .configure(Config {
            port: 16969,
            address: "0.0.0.0".parse().unwrap(),
            ..Default::default()
        })
}