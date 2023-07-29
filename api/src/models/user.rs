use super::password::Password;
use crate::schema::users;
use diesel::{prelude::*, AsExpression, sql_types::Integer};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum_macros::{EnumIter, EnumString};

#[derive(Queryable, Insertable, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: Password,
    pub role: Role,
    pub confirmed: bool,
    pub created_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, EnumString, EnumIter, PartialEq, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum Role {
    User,
    Admin,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
