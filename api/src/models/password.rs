use std::io::Write;

use diesel::{
    backend::Backend, deserialize::FromSql, expression::AsExpression, pg::Pg, serialize::ToSql,
    sql_types::Text, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, Default, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub struct Password {
    hash: String,
    salt: String,
}

impl Password {
    pub fn new(password: String) -> Self {
        let mut rng = urandom::csprng();
        let salt: [u8; 16] = rng.next();
        let salt = salt.iter().map(|x| format!("{:x}", x)).collect::<String>();
        let salty_password = password + &salt;
        let mut hasher = Sha256::new();
        hasher.update(&salty_password);
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        Self { hash, salt }
    }

    pub fn verify(&self, password: String) -> bool {
        let salty_password = password + &self.salt;
        let mut hasher = Sha256::new();
        hasher.update(&salty_password);
        let result = hasher.finalize();
        let hash = format!("{:x}", result);
        self.hash == hash
    }
}

impl ToSql<Text, Pg> for Password {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all((self.hash.clone() + ":" + &self.salt).as_bytes())?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl FromSql<Text, Pg> for Password {
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let all = String::from_sql(bytes)?;
        let all = all.split(|x| x == ':').collect::<Vec<&str>>();

        if all.len() != 2 {
            return Err(Box::new(diesel::result::Error::DeserializationError(
                "Invalid password format".into(),
            )));
        }

        let (hash, salt) = match (all[0], all[1]) {
            ref x if x.0.is_empty() || x.1.is_empty() => {
                return Err(Box::new(diesel::result::Error::DeserializationError(
                    "Invalid password format".into(),
                )))
            }
            _ => (all[0].to_string(), all[1].to_string()),
        };

        Ok(Self { hash, salt })
    }
}
