use super::{credentials::Credentials, password::Password, role::Role};
use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Queryable, Insertable, Serialize, Deserialize, Clone)]
pub struct User {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: Password,
    pub role: Role,
    pub confirmed: bool,
    #[diesel(deserialize_as = chrono::NaiveDateTime)]
    pub created_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl From<Credentials> for User {
    fn from(value: Credentials) -> Self {
        Self {
            name: value.name,
            password: Password::new(value.password),
            ..Default::default()
        }
    }
}
