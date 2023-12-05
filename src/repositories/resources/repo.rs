use crate::{models::resource_data::Resource, schema::resources, PgPool};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use petompp_web_models::{
    error::{Error, ResourceDataValidationError, ValidationError},
    models::country::Country,
};
use rocket::{async_trait, http::Status, outcome::Outcome, request::FromRequest, Request};

pub trait ResourcesRepo: Send + Sync {
    fn get(&self, key: &str, lang: &Country) -> Result<(Country, String), Error>;
    fn get_all(&self) -> Result<Vec<Resource>, Error>;
    fn create(&self, data: &Resource) -> Result<Resource, Error>;
    fn update(&self, data: &Resource) -> Result<Resource, Error>;
    fn delete(&self, key: &str) -> Result<(), Error>;
    fn delete_lang(&self, key: &str, lang: &Country) -> Result<(), Error>;
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
    fn get(&self, key: &str, lang: &Country) -> Result<(Country, String), Error> {
        let mut conn = self.get()?;
        let q = resources::dsl::resources.filter(resources::key.eq(key));
        Ok(match lang {
            Country::Poland => q
                .select((resources::pl, resources::en))
                .get_result::<(Option<String>, String)>(&mut conn)
                .map(|(pl, en)| match pl {
                    Some(pl) => (Country::Poland, pl),
                    None => (Country::UnitedKingdom, en),
                })?,
            _ => q
                .select(resources::en)
                .get_result::<String>(&mut conn)
                .map(|en| (Country::UnitedKingdom, en))?,
        })
    }

    fn get_all(&self) -> Result<Vec<Resource>, Error> {
        let mut conn = self.get()?;
        let res = resources::dsl::resources.load::<Resource>(&mut conn)?;
        Ok(res)
    }

    fn create(&self, data: &Resource) -> Result<Resource, Error> {
        let mut conn = self.get()?;
        let res = diesel::insert_into(resources::dsl::resources)
            .values(data)
            .get_result::<Resource>(&mut conn)?;
        Ok(res)
    }

    fn update(&self, data: &Resource) -> Result<Resource, Error> {
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
                .get_result::<Resource>(&mut conn)?,
            (Some(en), None) => diesel::update(resources::dsl::resources)
                .filter(resources::dsl::key.eq(key))
                .set(resources::dsl::en.eq(en))
                .get_result::<Resource>(&mut conn)?,
            (None, Some(pl)) => diesel::update(resources::dsl::resources)
                .filter(resources::dsl::key.eq(key))
                .set(resources::dsl::pl.eq(pl))
                .get_result::<Resource>(&mut conn)?,
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

    fn delete_lang(&self, key: &str, lang: &Country) -> Result<(), Error> {
        let mut conn = self.get()?;
        match lang {
            Country::UnitedKingdom => return Err(Error::ValidationError(ValidationError::Country)),
            Country::Poland => diesel::update(resources::dsl::resources)
                .filter(resources::dsl::key.eq(key))
                .set(resources::dsl::pl.eq::<Option<String>>(None))
                .execute(&mut conn)?,
        };
        Ok(())
    }
}
