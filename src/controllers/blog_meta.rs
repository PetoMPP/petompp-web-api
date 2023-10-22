use super::controller::Controller;
use crate::services::azure_blob::AzureBlobService;
use petompp_web_models::{
    error::{ApiError, Error, ValidationError},
    models::{api_response::ApiResponse, blog_data::BlogMetaData, country::Country},
};
use rocket::{get, routes, serde::json::Json, State};

pub struct BlogMetaController;

impl Controller for BlogMetaController {
    fn path(&self) -> &'static str {
        "/blog/meta"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![get, get_all]
    }
}

#[get("/<name>/<lang>")]
async fn get<'a>(
    name: &'a str,
    lang: &'a str,
    blob_service: &'a State<AzureBlobService>,
) -> Result<Json<ApiResponse<'a, BlogMetaData>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        blob_service
            .get_blog_meta(
                &name.to_string(),
                &Country::try_from(lang).map_err(|_| {
                    ApiError::from(Error::ValidationError(
                        ValidationError::Country,
                    ))
                })?,
            )
            .await?,
    )))
}

#[get("/")]
async fn get_all<'a>(
    blob_service: &'a State<AzureBlobService>,
) -> Result<Json<ApiResponse<'a, Vec<BlogMetaData>>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        blob_service.get_all_blog_meta().await?,
    )))
}
