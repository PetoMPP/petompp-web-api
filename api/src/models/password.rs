use std::io::Write;

use diesel::{
    backend::Backend, deserialize::FromSql, expression::AsExpression, pg::Pg, serialize::ToSql,
    sql_types::Text,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, Default, AsExpression)]
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
        let all = bytes
            .as_bytes()
            .split(|x| *x == b':')
            .map(|b| String::from_utf8(b.to_vec()).unwrap_or_default())
            .collect::<Vec<String>>();

        if all.len() != 2 {
            return Err(Box::new(diesel::result::Error::DeserializationError(
                "Invalid password format".into(),
            )));
        }

        let (hash, salt) = match (all[0].as_str(), all[1].as_str()) {
            ref x if x.0 == "" || x.1 == "" => {
                return Err(Box::new(diesel::result::Error::DeserializationError(
                    "Invalid password format".into(),
                )))
            }
            _ => (all[0].clone(), all[1].clone()),
        };

        Ok(Self { hash, salt })
    }
}
