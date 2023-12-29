use super::query::UsersQuery;
use crate::{models::user::User, repositories::query_config::QueryConfig, schema::users, PgPool};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use petompp_web_models::error::{Error, UserError};
use rocket::{async_trait, http::Status, outcome::Outcome, request::FromRequest, Request};

pub trait UserRepo: Send + Sync {
    fn create(&self, user: &User) -> Result<User, Error>;
    fn get_by_name(&self, normalized_name: String) -> Result<Option<User>, Error>;
    fn get_by_id(&self, id: i32) -> Result<Option<User>, Error>;
    fn get_all(&self, query_config: &QueryConfig) -> Result<Vec<Vec<User>>, Error>;
    fn activate(&self, id: i32) -> Result<Option<User>, Error>;
    fn delete(&self, id: i32) -> Result<Option<User>, Error>;
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r dyn UserRepo {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        let pool = request
            .guard::<&rocket::State<&dyn UserRepo>>()
            .await
            .unwrap();
        Outcome::Success(*pool.inner())
    }
}

impl UserRepo for PgPool {
    fn create(&self, user: &User) -> Result<User, Error> {
        let mut conn = self.get()?;
        let user = diesel::insert_into(users::dsl::users)
            .values(user)
            .get_result::<User>(&mut conn)
            .map_err(|e| unique_vol_as_user_exists(e, &*user.name))?;
        Ok(user)
    }

    fn get_by_name(&self, normalized_name: String) -> Result<Option<User>, Error> {
        let mut conn = self.get()?;
        Ok(users::dsl::users
            .filter(users::normalized_name.eq(&normalized_name))
            .first::<User>(&mut conn)
            .optional()?)
    }

    fn get_by_id(&self, id: i32) -> Result<Option<User>, Error> {
        let mut conn = self.get()?;
        Ok(users::dsl::users
            .filter(users::id.eq(id))
            .first::<User>(&mut conn)
            .optional()?)
    }

    fn get_all(&self, query_config: &QueryConfig) -> Result<Vec<Vec<User>>, Error> {
        let mut conn = self.get()?;
        Ok(vec![query_config
            .get_query()?
            .get_results::<User>(&mut conn)?])
    }

    fn activate(&self, id: i32) -> Result<Option<User>, Error> {
        let mut conn = self.get()?;
        Ok(diesel::update(users::dsl::users.filter(users::id.eq(id)))
            .set(users::confirmed.eq(true))
            .get_result::<User>(&mut conn)
            .optional()?)
    }

    fn delete(&self, id: i32) -> Result<Option<User>, Error> {
        let mut conn = self.get()?;
        Ok(diesel::update(users::dsl::users.filter(users::id.eq(id)))
            .set(users::deleted_at.eq(chrono::Utc::now().naive_utc()))
            .get_result::<User>(&mut conn)
            .optional()?)
    }
}

fn unique_vol_as_user_exists(e: diesel::result::Error, name: impl Into<String>) -> Error {
    match e {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => Error::User(UserError::NameTaken(name.into())),
        e => e.into(),
    }
}
