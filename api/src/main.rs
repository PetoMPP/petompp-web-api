use rocket::Config;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
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
