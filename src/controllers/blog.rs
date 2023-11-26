use super::controller::Controller;
use crate::{auth::claims::AdminClaims, services::azure_blob::AzureBlobService};
use petompp_web_models::{
    error::{ApiError, Error, ValidationError},
    models::{
        api_response::ApiResponse,
        blog_data::{BlogData, BlogMetaData},
        country::Country,
    },
};
use rocket::{get, post, routes, serde::json::Json, State};

pub struct BlogController;

impl Controller for BlogController {
    fn path(&self) -> &'static str {
        "/blog"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![create_or_update, get_meta, get_meta_all]
    }
}

#[post("/<name>/<lang>", data = "<value>")]
async fn create_or_update<'a>(
    _claims: AdminClaims,
    name: &'a str,
    lang: &'a str,
    blob_service: &'a State<AzureBlobService>,
    value: Json<BlogData>,
) -> Result<Json<ApiResponse<'a, &'a str>>, ApiError<'a>> {
    blob_service
        .create_or_update_blog_post(
            &name.to_string(),
            &Country::try_from(lang)
                .map_err(|_| ApiError::from(Error::ValidationError(ValidationError::Country)))?,
            &value.into_inner(),
        )
        .await?;
    Ok(Json(ApiResponse::ok("ok")))
}

#[get("/meta/<name>/<lang>")]
async fn get_meta<'a>(
    name: &'a str,
    lang: &'a str,
    blob_service: &'a State<AzureBlobService>,
) -> Result<Json<ApiResponse<'a, BlogMetaData>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        blob_service
            .get_blog_meta(
                &name.to_string(),
                &Country::try_from(lang).map_err(|_| {
                    ApiError::from(Error::ValidationError(ValidationError::Country))
                })?,
            )
            .await?,
    )))
}

#[get("/meta")]
async fn get_meta_all<'a>(
    blob_service: &'a State<AzureBlobService>,
) -> Result<Json<ApiResponse<'a, Vec<BlogMetaData>>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        blob_service.get_all_blog_meta().await?,
    )))
}
