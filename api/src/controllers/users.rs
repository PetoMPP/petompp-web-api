use rocket::{post, routes};

use crate::controllers::controller::Controller;

pub struct UsersController;

impl Controller for UsersController {
    fn path(&self) -> &'static str {
        "/users"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![create, login]
    }
}

#[post("/")]
fn create() -> &'static str {
    "Hello, world!"
}

#[post("/login")]
fn login() -> &'static str {
    "Hello, world!"
}
