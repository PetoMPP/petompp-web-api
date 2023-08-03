use super::repo::{RepoError, UserRepo};
use crate::{
    auth::token::create_token,
    models::{credentials::Credentials, user::User},
    schema::users,
    PgPool, Secrets,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

impl UserRepo for PgPool {
    fn create(&self, credentials: Credentials) -> Result<User, RepoError> {
        let mut conn = self.get()?;
        if users::dsl::users
            .filter(users::name.eq(&credentials.name))
            .count()
            .get_result::<i64>(&mut conn)?
            > 0
        {
            return Err(RepoError::UserAlreadyExists(credentials.name));
        }

        Ok(diesel::insert_into(users::dsl::users)
            .values(&User::from(credentials))
            .get_result::<User>(&mut conn)?)
    }

    fn login(&self, credentials: Credentials, secrets: &Secrets) -> Result<String, RepoError> {
        let mut conn = self.get()?;
        let Some(user) = users::dsl::users
            .filter(users::name.eq(&credentials.name))
            .first::<User>(&mut conn)
            .optional()? else {
            return Err(RepoError::UserNotFound(credentials.name));
        };
        if user.confirmed {
            return Err(RepoError::UserNotConfirmed(credentials.name));
        }
        if !user.password.verify(credentials.password.clone()) {
            return Err(RepoError::InvalidCredentials);
        }
        Ok(create_token(secrets, &user)?)
    }

    fn get_self(&self, user_id: i32) -> Result<User, RepoError> {
        let mut conn = self.get()?;
        let Some(user) = users::dsl::users
            .filter(users::id.eq(user_id))
            .first::<User>(&mut conn)
            .optional()? else {
            return Err(RepoError::UserNotFound(format!("ID: {}", user_id)));
        };
        Ok(user)
    }

    fn activate(&self, user_id: i32) -> Result<User, RepoError> {
        let mut conn = self.get()?;
        let Ok(user) = diesel::update(users::dsl::users.filter(users::id.eq(user_id)))
            .set(users::confirmed.eq(true))
            .get_result::<User>(&mut conn) else {
            return Err(RepoError::UserNotFound(format!("ID: {}", user_id)));
        };
        Ok(user)
    }
}
