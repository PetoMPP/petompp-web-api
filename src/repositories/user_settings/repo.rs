use crate::{models::user_settings::UserSettings, schema::user_settings, PgPool};
use diesel::RunQueryDsl;
use petompp_web_models::error::Error;
use rocket::{async_trait, http::Status, outcome::Outcome, request::FromRequest, Request};

pub trait UserSettingsRepo: Send + Sync {
    fn get(&self) -> Result<UserSettings, Error>;
    fn update(&self, settings: &UserSettings) -> Result<UserSettings, Error>;
}

#[async_trait]
impl<'r> FromRequest<'r> for &'r dyn UserSettingsRepo {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        let pool = request
            .guard::<&rocket::State<&dyn UserSettingsRepo>>()
            .await
            .unwrap();
        Outcome::Success(*pool.inner())
    }
}

impl UserSettingsRepo for PgPool {
    fn get(&self) -> Result<UserSettings, Error> {
        let mut conn = self.get()?;
        Ok(user_settings::dsl::user_settings.first::<UserSettings>(&mut conn)?)
    }

    fn update(&self, settings: &UserSettings) -> Result<UserSettings, Error> {
        let mut conn = self.get()?;
        Ok(diesel::update(user_settings::dsl::user_settings)
            .set(settings)
            .get_result::<UserSettings>(&mut conn)?)
    }
}
