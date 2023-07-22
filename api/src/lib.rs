use std::env;

pub mod auth;
pub mod controllers;
pub mod extensions;

pub struct Secrets {
    pub api_secret: String,
}

impl Default for Secrets {
    fn default() -> Self {
        let api_secret = env::var("API_SECRET").unwrap_or("shhhdonttellanyoneaboutit".to_string());
        Self { api_secret }
    }
}
