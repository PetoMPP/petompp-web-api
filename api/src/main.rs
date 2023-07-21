use petompp_web_api::{
    controllers::{controller::ControllerRegisterer, users::UsersController},
    extensions::extension::Extension,
};
use rocket::Config;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    Extension(rocket::build())
        .add(UsersController)
        .into()
        .mount("/", routes![index])
        .configure(Config {
            port: 16969,
            address: "0.0.0.0".parse().unwrap(),
            ..Default::default()
        })
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
