use super::controller::Controller;
use crate::{
    auth::claims::AdminClaims, models::resource_data::Resource,
    repositories::resources::repo::ResourcesRepo,
};
use petompp_web_models::{
    error::{ApiError, Error, ResourceDataValidationError, ValidationError},
    models::{api_response::ApiResponse, country::Country, resource_data::ResourceData},
};
use rocket::{delete, get, post, put, routes, serde::json::Json};

pub struct ResourcesController;

impl Controller for ResourcesController {
    fn path(&self) -> &'static str {
        "/res"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![get, get_all_keys, create, update, delete, delete_lang]
    }
}

#[get("/<key>?<lang>")]
async fn get<'a>(
    key: &'a str,
    lang: Country,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, (Country, String)>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(pool.get(key, &lang)?)))
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
    let value = Resource {
        key: Some(key.to_string()),
        ..value.into_inner().into()
    };
    Ok(Json(ApiResponse::ok(pool.create(&value)?.into())))
}

#[post("/<key>", data = "<value>")]
async fn update<'a>(
    _admin_claims: AdminClaims,
    key: &'a str,
    value: Json<ResourceData>,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, ResourceData>>, ApiError<'a>> {
    if key != value.key.as_str() {
        return Err(Error::ValidationError(ValidationError::ResourceData(
            ResourceDataValidationError::KeyMismatch(key.to_string(), value.key.clone()),
        ))
        .into());
    }
    let value = Resource {
        key: Some(key.to_string()),
        ..value.into_inner().into()
    };
    Ok(Json(ApiResponse::ok(pool.update(&value)?.into())))
}

#[delete("/<key>")]
async fn delete<'a>(
    _admin_claims: AdminClaims,
    key: &'a str,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, &'a str>>, ApiError<'a>> {
    pool.delete(key)?;
    Ok(Json(ApiResponse::ok("ok")))
}

#[delete("/<key>?<lang>")]
async fn delete_lang<'a>(
    _admin_claims: AdminClaims,
    key: &'a str,
    lang: Country,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, &'a str>>, ApiError<'a>> {
    pool.delete_lang(key, &lang)?;
    Ok(Json(ApiResponse::ok("ok")))
}
