use super::controller::Controller;
use crate::{auth::claims::AdminClaims, repositories::user_settings::repo::UserSettingsRepo};
use petompp_web_models::{
    error::ApiError,
    models::{api_response::ApiResponse, user_settings_dto::UserSettingsDto},
};
use rocket::{get, post, routes, serde::json::Json};

pub struct UserSettingsController;

impl Controller for UserSettingsController {
    fn path(&self) -> &'static str {
        "/settings/users"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![get, update]
    }
}

#[get("/")]
async fn get(pool: &dyn UserSettingsRepo) -> Result<Json<ApiResponse<UserSettingsDto>>, ApiError> {
    let settings = pool.get()?;
    Ok(Json(ApiResponse::ok(settings.into())))
}

#[post("/", data = "<settings>")]
async fn update(
    _claims: AdminClaims,
    settings: Json<UserSettingsDto>,
    pool: &dyn UserSettingsRepo,
) -> Result<Json<ApiResponse<UserSettingsDto>>, ApiError> {
    let settings = pool.update(&settings.into_inner().into())?;
    Ok(Json(ApiResponse::ok(settings.into())))
}
