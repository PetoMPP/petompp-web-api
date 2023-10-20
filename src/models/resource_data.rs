use diesel::{Insertable, Queryable};
use petompp_web_models::models::resource_data::ResourceData;

#[derive(Default, Queryable, Insertable, Clone)]
#[diesel(table_name = crate::schema::resources)]
pub struct Resource {
    #[diesel(deserialize_as = String)]
    pub key: Option<String>,
    #[diesel(deserialize_as = String)]
    pub en: Option<String>,
    pub pl: Option<String>,
}

impl Into<ResourceData> for Resource {
    fn into(self) -> ResourceData {
        ResourceData {
            key: self.key.unwrap(),
            en: self.en,
            pl: self.pl,
        }
    }
}

impl From<ResourceData> for Resource {
    fn from(data: ResourceData) -> Self {
        Self {
            key: Some(data.key),
            en: data.en,
            pl: data.pl,
        }
    }
}
