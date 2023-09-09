use crate::{
    error::{Error, ResourceDataValidationError, ValidationError},
    models::resource_data::ResourceData,
    schema::resources,
    PgPool,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rocket::{async_trait, http::Status, outcome::Outcome, request::FromRequest, Request};

pub trait ResourcesRepo: Send + Sync {
    fn get(&self, key: &str, lang: &str) -> Result<String, Error>;
    fn get_all(&self) -> Result<Vec<ResourceData>, Error>;
    fn create(&self, data: &ResourceData) -> Result<ResourceData, Error>;
    fn update(&self, data: &ResourceData) -> Result<ResourceData, Error>;
    fn delete(&self, key: &str) -> Result<(), Error>;
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
    fn get(&self, key: &str, lang: &str) -> Result<String, Error> {
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

    fn get_all(&self) -> Result<Vec<ResourceData>, Error> {
        let mut conn = self.get()?;
        let res = resources::dsl::resources.load::<ResourceData>(&mut conn)?;
        Ok(res)
    }

    fn create(&self, data: &ResourceData) -> Result<ResourceData, Error> {
        let mut conn = self.get()?;
        let res = diesel::insert_into(resources::dsl::resources)
            .values(data)
            .get_result::<ResourceData>(&mut conn)?;
        Ok(res)
    }

    fn update(&self, data: &ResourceData) -> Result<ResourceData, Error> {
        let mut conn = self.get()?;
        let key = data
            .key
            .clone()
            .ok_or(Error::ValidationError(ValidationError::ResourceData(
                ResourceDataValidationError::KeyMissing,
            )))?;
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
            _ => {
                return Err(Error::ValidationError(ValidationError::ResourceData(
                    ResourceDataValidationError::ValueMissing,
                )))
            }
        };
        Ok(res)
    }

    fn delete(&self, key: &str) -> Result<(), Error> {
        let mut conn = self.get()?;
        diesel::delete(resources::dsl::resources.filter(resources::dsl::key.eq(key)))
            .execute(&mut conn)?;
        Ok(())
    }
}
