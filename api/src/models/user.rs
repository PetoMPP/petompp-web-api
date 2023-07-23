use std::fmt::Display;

use super::password::Password;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: Password,
    pub role: Role,
    pub confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, EnumString, EnumIter, PartialEq)]
pub enum Role {
    User,
    Admin,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
