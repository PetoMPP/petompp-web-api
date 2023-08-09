use crate::repositories::repo::RepoError;
use deref_derive::{Deref, DerefMut};
use diesel::{
    backend::Backend, deserialize::FromSql, pg::Pg, serialize::ToSql, sql_types::Text,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, AsExpression, FromSqlRow, Deref, DerefMut,
)]
#[diesel(sql_type = Text)]
pub struct UserName(String);

impl UserName {
    pub fn new(name: String) -> Result<Self, RepoError> {
        let name = name.trim();
        validate_name(&name)?;
        Ok(Self(name.to_string()))
    }
}

fn validate_name(name: &str) -> Result<(), RepoError> {
    const MIN_LENGTH: usize = 3;
    const MAX_LENGTH: usize = 28;
    const SPECIAL_CHARS: [char; 11] = ['-', '_', '.', '$', '@', '!', '#', '%', '^', '&', '*'];

    if !(MIN_LENGTH..MAX_LENGTH).contains(&name.len()) {
        return Err(RepoError::ValidationError(format!(
            "Name must be between {} and {} characters long.",
            MIN_LENGTH, MAX_LENGTH
        )));
    }
    
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || SPECIAL_CHARS.contains(&c))
    {
        return Err(RepoError::ValidationError(
            "Name must be alphanumeric with special characters: - _ . $ @ ! # % ^ & *".to_string(),
        ));
    }
    Ok(())
}

impl ToSql<Text, Pg> for UserName {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(self.0.as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, Pg> for UserName {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        Ok(Self(String::from_sql(bytes)?))
    }
}
