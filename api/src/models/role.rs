use diesel::{
    deserialize::FromSql, pg::Pg, serialize::ToSql, sql_types::Integer, AsExpression, FromSqlRow,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::Write};
use strum_macros::{EnumIter, EnumString};

#[derive(
    Default,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    EnumString,
    EnumIter,
    PartialEq,
    AsExpression,
    FromPrimitive,
    FromSqlRow,
)]
#[diesel(sql_type = Integer)]
pub enum Role {
    #[default]
    User,
    Admin,
}

impl ToSql<Integer, Pg> for Role {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(&(*self as i32).to_ne_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Integer, Pg> for Role {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        FromPrimitive::from_i32(i32::from_sql(bytes)?).ok_or(Box::new(
            diesel::result::Error::DeserializationError("Invalid role".into()),
        ))
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}
