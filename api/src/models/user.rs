use super::password::Password;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: Password,
}
