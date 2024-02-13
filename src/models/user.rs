use super::{password::Password, role::Role, user_name::UserName};
use crate::schema::users;
use diesel::prelude::*;
use petompp_web_models::models::user::UserData;

#[derive(Default, Queryable, Insertable, Clone)]
pub struct User {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: UserName,
    pub normalized_name: String,
    pub password: Password,
    pub role: Role,
    pub confirmed: bool,
    #[diesel(deserialize_as = chrono::NaiveDateTime)]
    pub created_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl User {
    pub fn new(name: String, password: String, role: Role) -> Self {
        let name = name.trim();
        let name = UserName(name.to_string());
        let normalized_name = name.to_lowercase();
        let password = Password::new(password);
        Self {
            name,
            normalized_name,
            password,
            role,
            ..Default::default()
        }
    }
}

impl From<User> for UserData {
    fn from(val: User) -> Self {
        UserData {
            id: val.id.unwrap(),
            name: val.name.0.clone(),
            role: val.role.into(),
            confirmed: val.confirmed,
            created_at: val.created_at.unwrap(),
            deleted_at: val.deleted_at,
        }
    }
}
