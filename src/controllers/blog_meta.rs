use super::controller::Controller;
use crate::services::azure_blob::AzureBlobService;
use petompp_web_models::{
    error::ApiError,
    models::{api_response::ApiResponse, blog_data::BlogMetaData},
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

#[get("/<name>")]
async fn get<'a>(
    name: &'a str,
    blob_service: &'a State<AzureBlobService>,
) -> Result<Json<ApiResponse<'a, BlogMetaData>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        blob_service.get_blog_meta(name.to_string()).await?,
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
