use super::{password::Password, role::Role, user_name::UserName};
use crate::schema::users;
use diesel::prelude::*;
use petompp_web_models::{error::Error, models::user::UserData};

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
    pub fn new(name: String, password: String, role: Role) -> Result<Self, Error> {
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

impl Into<UserData> for User {
    fn into(self) -> UserData {
        UserData {
            id: self.id.unwrap(),
            name: self.name.0.clone(),
            role: self.role.into(),
            confirmed: self.confirmed,
            created_at: self.created_at.unwrap(),
            deleted_at: self.deleted_at,
        }
    }
}
