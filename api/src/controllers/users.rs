use crate::{
    auth::{
        claims::{AdminClaims, Claims},
        token::create_token,
    },
    controllers::controller::Controller,
    models::{credentials::Credentials, user::User},
    schema::users,
    PgPool,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use rocket::{get, http::Status, post, response::status, routes, serde::json::Json, Build, State};

pub struct UsersController;

impl Controller for UsersController {
    fn path(&self) -> &'static str {
        "/users"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![create, login, get_self, activate]
    }

    fn add_managed(&self, rocket: rocket::Rocket<Build>) -> rocket::Rocket<Build> {
        rocket
    }
}

#[post("/", data = "<credentials>")]
async fn create(credentials: Json<Credentials>, pool: &State<PgPool>) -> status::Custom<String> {
    let Ok(mut conn) = pool.get() else {
        return status::Custom(Status::RequestTimeout, "Database connection error!".to_string());
    };
    let Ok(user_count) = users::dsl::users
        .filter(users::name.eq(&credentials.name))
        .count()
        .get_result::<i64>(&mut conn) else
    {
        return status::Custom(Status::InternalServerError, "Database error!".to_string());
    };

    if user_count > 0 {
        return status::Custom(
            Status::BadRequest,
            format!("User {} already exists!", credentials.name),
        );
    }

    match diesel::insert_into(users::dsl::users)
        .values(&User::from(credentials.into_inner()))
        .get_result::<User>(&mut conn)
    {
        Ok(user) => status::Custom(Status::Ok, serde_json::to_string_pretty(&user).unwrap()),
        Err(e) => status::Custom(Status::InternalServerError, e.to_string()),
    }
}

#[post("/login", data = "<credentials>")]
async fn login(
    credentials: Json<Credentials>,
    pool: &State<PgPool>,
    secrets: &State<crate::Secrets>,
) -> status::Custom<String> {
    let Ok(mut conn) = pool.get() else {
        return status::Custom(Status::RequestTimeout, "Database connection error!".to_string());
    };

    let Ok(user) = users::dsl::users
        .filter(users::name.eq(&credentials.name))
        .first::<User>(&mut conn)
        .optional() else {
        return status::Custom(Status::InternalServerError, "Database error!".to_string());
    };

    let Some(user) = user else {
        return status::Custom(Status::BadRequest, "User does not exist!".to_string());
    };

    if !user.confirmed {
        return status::Custom(Status::BadRequest, "User not activated!".to_string());
    }
    if !user.password.verify(credentials.password.clone()) {
        return status::Custom(Status::BadRequest, "Wrong password!".to_string());
    }
    return match create_token(secrets, &user) {
        Ok(token) => status::Custom(Status::Ok, token),
        Err(e) => status::Custom(Status::InternalServerError, e.to_string()),
    };
}

#[get("/")]
async fn get_self(claims: Claims, pool: &State<PgPool>) -> status::Custom<String> {
    let Ok(mut conn) = pool.get() else {
        return status::Custom(Status::RequestTimeout, "Database connection error!".to_string());
    };

    let Ok(user) = users::dsl::users
        .filter(users::id.eq(claims.sub))
        .first::<User>(&mut conn)
        .optional() else {
        return status::Custom(Status::InternalServerError, "Database error!".to_string());
    };

    match user {
        Some(user) => status::Custom(Status::Ok, serde_json::to_string_pretty(&user).unwrap()),
        None => status::Custom(Status::NotFound, "User does not exist!".to_string()),
    }
}

#[post("/<id>/activate")]
async fn activate(
    _claims: AdminClaims,
    id: i32,
    pool: &State<PgPool>,
) -> status::Custom<&'static str> {
    let Ok(mut conn) = pool.get() else {
        return status::Custom(Status::RequestTimeout, "Database connection error!");
    };

    let Ok(affected) = diesel::update(users::dsl::users.filter(users::id.eq(id)))
        .set(users::confirmed.eq(true))
        .execute(&mut conn) else {
            return status::Custom(Status::InternalServerError, "Database error!");
    };

    if affected == 0 {
        return status::Custom(Status::NotFound, "User not found!");
    }
    status::Custom(Status::Ok, "User activated!")
}
