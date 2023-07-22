use crate::{
    auth::validation::{create_token, Claims},
    controllers::controller::Controller,
    models::{credentials::Credentials, password::Password, user::User},
};
use rocket::{
    futures::lock::Mutex, get, http::Status, post, response::status, routes, serde::json::Json,
    Build, State,
};

pub struct UserData(pub Mutex<Vec<User>>);

pub struct UsersController;

impl Controller for UsersController {
    fn path(&self) -> &'static str {
        "/users"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![create, login, get_self]
    }

    fn add_managed(&self, rocket: rocket::Rocket<Build>) -> rocket::Rocket<Build> {
        rocket.manage(UserData(Mutex::new(Vec::new())))
    }
}

#[post("/", data = "<credentials>")]
async fn create(
    credentials: Json<Credentials>,
    user_data: &State<UserData>,
) -> status::Custom<&'static str> {
    let mut user_mutex = user_data.0.lock().await;
    return match user_mutex.iter().find(|user| user.name == credentials.name) {
        Some(_) => status::Custom(Status::BadRequest, "User already exists!"),
        None => {
            let id = user_mutex.len() as u32 + 1;
            user_mutex.push(User {
                id,
                name: credentials.name.clone(),
                password: Password::new(credentials.password.clone()),
            });
            status::Custom(Status::Ok, "User created!")
        }
    };
}

#[post("/login", data = "<credentials>")]
async fn login(
    credentials: Json<Credentials>,
    user_data: &State<UserData>,
    secrets: &State<crate::Secrets>,
) -> status::Custom<String> {
    let user_mutex = user_data.0.lock().await;
    return match user_mutex.iter().find(|user| user.name == credentials.name) {
        Some(user) => {
            if user.password.verify(credentials.password.clone()) {
                return match create_token(secrets, user) {
                    Ok(token) => status::Custom(Status::Ok, token),
                    Err(e) => status::Custom(Status::InternalServerError, e.to_string()),
                };
            } else {
                status::Custom(Status::BadRequest, "Wrong password!".to_string())
            }
        }
        None => status::Custom(Status::BadRequest, "User does not exist!".to_string()),
    };
}

#[get("/")]
async fn get_self(
    claims: Claims,
    user_data: &State<UserData>,
) -> status::Custom<Option<Json<User>>> {
    let user_mutex = user_data.0.lock().await;
    return match user_mutex.iter().find(|u| u.id == claims.sub) {
        Some(user) => status::Custom(Status::Ok, Some(Json(user.clone()))),
        None => status::Custom(Status::NotFound, None),
    };
}
