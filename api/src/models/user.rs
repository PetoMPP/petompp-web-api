use super::{password::Password, role::Role, user_name::UserName};
use crate::{repositories::repo::RepoError, schema::users};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Queryable, Insertable, Serialize, Deserialize, Clone)]
pub struct User {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: UserName,
    pub normalized_name: String,
    #[serde(skip_serializing)]
    pub password: Password,
    pub role: Role,
    pub confirmed: bool,
    #[diesel(deserialize_as = chrono::NaiveDateTime)]
    pub created_at: Option<chrono::NaiveDateTime>,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl User {
    pub fn new(name: String, password: String, role: Role) -> Result<Self, RepoError> {
        let name = UserName::new(name)?;
        let normalized_name = name.to_lowercase();
        let password = Password::new(password)?;
        Ok(Self {
            name,
            normalized_name,
            password,
            role,
            ..Default::default()
        })
    }
}
