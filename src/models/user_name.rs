use deref_derive::{Deref, DerefMut};
use diesel::{
    backend::Backend, deserialize::FromSql, pg::Pg, serialize::ToSql, sql_types::Text,
    AsExpression, FromSqlRow,
};
use std::io::Write;

#[derive(Debug, Clone, Default, AsExpression, FromSqlRow, Deref, DerefMut)]
#[diesel(sql_type = Text)]
pub struct UserName(pub String);

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
