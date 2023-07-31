use chrono::{NaiveDate, NaiveTime};
use petompp_web_api::{
    auth::token::create_token,
    models::{password::Password, role::Role, user::User},
    Secrets,
};
use rocket::{http::Header, local::blocking::Client};
use strum::IntoEnumIterator;

#[test]
fn activate_test() {
    let rocket = petompp_web_api::build_rocket();
    let client = Client::untracked(rocket).unwrap();
    for role in Role::iter() {
        let user = User {
            id: Some(1),
            name: "admin".to_string(),
            password: Password::new("password".to_string()),
            role,
            confirmed: true,
            created_at: Some(chrono::NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 07, 29).unwrap(),
                NaiveTime::from_hms_opt(19, 05, 11).unwrap(),
            )),
            deleted_at: None,
        };
        let mut req = client.post("/api/v1/users/1/activate");
        req.add_header(Header::new(
            "Authorization",
            format!(
                "Bearer {}",
                create_token(&Secrets::default(), &user).unwrap()
            ),
        ));
        let expected = match role {
            Role::Admin => rocket::http::Status::Ok,
            _ => rocket::http::Status::Unauthorized,
        };

        let response = req.dispatch();

        assert_eq!(response.status(), expected);
    }
}

#[test]
fn activate_test_no_auth() {
    let rocket = petompp_web_api::build_rocket();
    let client = Client::untracked(rocket).unwrap();
    let req = client.post("/api/v1/users/1/activate");

    let response = req.dispatch();

    assert_eq!(response.status(), rocket::http::Status::Unauthorized);
}
