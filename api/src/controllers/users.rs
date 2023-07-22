use crate::{controllers::controller::Controller, auth::validation::create_token};
use rocket::{
    futures::lock::Mutex, http::Status, post, response::status, routes, serde::json::Json, Build,
    State,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
struct Credentials {
    name: String,
    password: String,
}

pub struct UserData(pub Mutex<Vec<User>>);

pub struct User {
    pub id: u32,
    pub name: String,
    pub password: Password,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Password {
    pub hash: String,
    pub salt: String,
}

impl Password {
    pub fn new(password: String) -> Self {
        let mut rng = urandom::csprng();
        let salt: [u8; 16] = rng.next();
        let salt = salt.iter().map(|x| format!("{:x}", x)).collect::<String>();
        let salty_password = password + &salt;
        let mut hasher = Sha256::new();
        hasher.update(&salty_password);
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        Self { hash, salt }
    }

    pub fn verify(&self, password: String) -> bool {
        let salty_password = password + &self.salt;
        let mut hasher = Sha256::new();
        hasher.update(&salty_password);
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        self.hash == hash
    }
}

pub struct UsersController;

impl Controller for UsersController {
    fn path(&self) -> &'static str {
        "/users"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![create, login]
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
                    Err(_) => status::Custom(Status::InternalServerError, "Token creation failed!".to_string()),
                };
            } else {
                status::Custom(Status::BadRequest, "Wrong password!".to_string())
            }
        }
        None => status::Custom(Status::BadRequest, "User does not exist!".to_string()),
    };
}
