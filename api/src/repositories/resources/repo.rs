use crate::{
    models::resource_data::ResourceData, repositories::repo::RepoError, schema::resources, PgPool,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{async_trait, http::Status, outcome::Outcome, request::FromRequest, Request};

pub trait ResourcesRepo: Send + Sync {
    fn get(&self, key: &str, lang: &str) -> Result<String, RepoError>;
    fn create(&self, data: &ResourceData) -> Result<ResourceData, RepoError>;
    fn update(&self, data: &ResourceData) -> Result<ResourceData, RepoError>;
    fn delete(&self, key: &str) -> Result<(), RepoError>;
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r dyn ResourcesRepo {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        request
            .guard::<&rocket::State<&dyn ResourcesRepo>>()
            .await
            .map(|pool| *pool.inner())
    }
}

impl ResourcesRepo for PgPool {
    fn get(&self, key: &str, lang: &str) -> Result<String, RepoError> {
        let mut conn = self.get()?;
        let q = resources::dsl::resources.filter(resources::key.eq(key));
        let res = match lang {
            "pl" => {
                let (pl, en) = q
                    .select((resources::pl, resources::en))
                    .get_result::<(Option<String>, String)>(&mut conn)?;
                pl.unwrap_or(en)
            }
            "en" | _ => q.select(resources::en).get_result::<String>(&mut conn)?,
        };
        Ok(res)
    }

    fn create(&self, data: &ResourceData) -> Result<ResourceData, RepoError> {
        let mut conn = self.get()?;
        let res = diesel::insert_into(resources::dsl::resources)
            .values(data)
            .get_result::<ResourceData>(&mut conn)?;
        Ok(res)
    }

    fn update(&self, data: &ResourceData) -> Result<ResourceData, RepoError> {
        let mut conn = self.get()?;
        let key = data
            .key
            .clone()
            .ok_or(RepoError::ValidationError("Key is required.".to_string()))?;
        let res = match (&data.en, &data.pl) {
            (Some(en), Some(pl)) => diesel::update(resources::dsl::resources)
                .filter(resources::dsl::key.eq(key))
                .set((resources::dsl::en.eq(en), resources::dsl::pl.eq(pl)))
                .get_result::<ResourceData>(&mut conn)?,
            (Some(en), None) => diesel::update(resources::dsl::resources)
                .filter(resources::dsl::key.eq(key))
                .set(resources::dsl::en.eq(en))
                .get_result::<ResourceData>(&mut conn)?,
            (None, Some(pl)) => diesel::update(resources::dsl::resources)
                .filter(resources::dsl::key.eq(key))
                .set(resources::dsl::pl.eq(pl))
                .get_result::<ResourceData>(&mut conn)?,
            _ => return Err(RepoError::ValidationError("Nothing to update.".to_string())),
        };
        Ok(res)
    }

    fn delete(&self, key: &str) -> Result<(), RepoError> {
        let mut conn = self.get()?;
        diesel::delete(resources::dsl::resources.filter(resources::dsl::key.eq(key)))
            .execute(&mut conn)?;
        Ok(())
    }
}
