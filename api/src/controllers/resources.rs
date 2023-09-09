use crate::{
    auth::claims::AdminClaims,
    controllers::response::ApiResponse,
    error::{ApiError, Error, ResourceDataValidationError, ValidationError},
    models::resource_data::ResourceData,
    repositories::resources::repo::ResourcesRepo,
};

use super::controller::Controller;
use rocket::{delete, get, post, put, routes, serde::json::Json, Build};

pub struct ResourcesController;

impl Controller for ResourcesController {
    fn path(&self) -> &'static str {
        "/res"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![get, get_all_keys, create, update, delete]
    }

    fn add_managed(&self, rocket: rocket::Rocket<Build>) -> rocket::Rocket<Build> {
        rocket
    }
}

#[get("/<key>?<lang>")]
async fn get<'a>(
    key: &'a str,
    lang: &'a str,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(pool.get(key, lang)?)))
}

#[get("/keys")]
async fn get_all_keys<'a>(
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, Vec<String>>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        pool.get_all()?
            .iter()
            .map(|x| x.key.clone().unwrap())
            .collect(),
    )))
}

#[put("/<key>", data = "<value>")]
async fn create<'a>(
    _admin_claims: AdminClaims,
    key: &'a str,
    value: Json<ResourceData>,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, ResourceData>>, ApiError<'a>> {
    let value = ResourceData {
        key: Some(key.to_string()),
        ..value.into_inner()
    };
    Ok(Json(ApiResponse::ok(pool.create(&value)?)))
}

#[post("/<key>", data = "<value>")]
async fn update<'a>(
    _admin_claims: AdminClaims,
    key: &'a str,
    value: Json<ResourceData>,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, ResourceData>>, ApiError<'a>> {
    if key != value.key.as_ref().unwrap().as_str() {
        return Err(Error::ValidationError(ValidationError::ResourceData(
            ResourceDataValidationError::KeyMismatch(
                key.to_string(),
                value.key.as_ref().unwrap().clone(),
            ),
        ))
        .into());
    }
    let value = ResourceData {
        key: Some(key.to_string()),
        ..value.into_inner()
    };
    Ok(Json(ApiResponse::ok(pool.update(&value)?)))
}

#[delete("/<key>")]
async fn delete(
    _admin_claims: AdminClaims,
    key: &str,
    pool: &dyn ResourcesRepo,
) -> Result<&'static str, ApiError<'static>> {
    pool.delete(key)?;
    Ok("OK")
}
