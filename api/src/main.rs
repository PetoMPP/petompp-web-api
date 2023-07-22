use petompp_web_api::{
    controllers::{controller::ControllerRegisterer, users::UsersController},
    extensions::extension::Extension, Secrets,
};
use rocket::Config;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
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
