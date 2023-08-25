use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Default, Queryable, Insertable, Serialize, Deserialize, Clone)]
#[diesel(table_name = crate::schema::resources)]
pub struct ResourceData {
    #[diesel(deserialize_as = String)]
    pub key: Option<String>,
    #[diesel(deserialize_as = String)]
    pub en: Option<String>,
    pub pl: Option<String>,
}
