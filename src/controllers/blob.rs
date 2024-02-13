use super::controller::Controller;
use crate::services::azure_blob::AzureBlobService;
use azure_storage_blobs::prelude::ClientBuilder;
use petompp_web_models::{
    error::ApiError,
    models::{
        api_response::ApiResponse,
        blob::blob_meta::{BlobMetaData, BlobMetaDto},
    },
};
use rocket::{
    delete,
    form::{Form, FromFormField},
    get,
    http::uri::{fmt::Path, Segments},
    post, routes,
    serde::json::Json,
    FromForm, State,
};
use serde::{Deserialize, Serialize};

pub struct BlobController;

impl Controller for BlobController {
    fn path(&self) -> &'static str {
        "/blob"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![get, get_all, create_or_update, delete]
    }
}

pub enum BlobDataMode {
    Full,
    Name,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum BlobsResponse {
    Full(Vec<BlobMetaData>),
    Name(Vec<String>),
}

impl<'a> FromFormField<'a> for BlobDataMode {
    fn from_value(field: rocket::form::ValueField<'a>) -> rocket::form::Result<'a, Self> {
        match field.value {
            "full" => Ok(BlobDataMode::Full),
            "name" => Ok(BlobDataMode::Name),
            _ => Err(rocket::form::Error::validation("Invalid blob data mode".to_string()).into()),
        }
    }
}

#[get("/<container>/<filename..>")]
async fn get<'a>(
    container: &'a str,
    filename: Segments<'a, Path>,
    blob_service: &'a State<ClientBuilder>,
) -> Result<Json<ApiResponse<'a, BlobMetaData>>, ApiError<'a>> {
    let filename = filename
        .into_iter()
        .fold(String::new(), |f, s| format!("{}/{}", f, s))[1..]
        .to_string();
    Ok(Json(ApiResponse::ok(
        blob_service
            .get(container.to_string(), filename.to_string())
            .await?,
    )))
}

#[get("/<container>?<data>&<prefix>")]
async fn get_all<'a>(
    container: &'a str,
    data: BlobDataMode,
    prefix: Option<&'a str>,
    blob_service: &'a State<ClientBuilder>,
) -> Result<Json<ApiResponse<'a, BlobsResponse>>, ApiError<'a>> {
    match data {
        BlobDataMode::Full => Ok(Json(ApiResponse::ok(BlobsResponse::Full(
            blob_service
                .get_all(container.to_string(), prefix.map(|s| s.to_string()))
                .await?,
        )))),
        BlobDataMode::Name => Ok(Json(ApiResponse::ok(BlobsResponse::Name(
            blob_service
                .get_all(container.to_string(), prefix.map(|s| s.to_string()))
                .await?
                .into_iter()
                .map(|b| b.filename)
                .collect(),
        )))),
    }
}

#[derive(FromForm)]
struct BlobUploadForm<'a> {
    meta: BlobMetaDto,
    content: &'a [u8],
}

#[post("/<container>", data = "<value>")]
async fn create_or_update<'a>(
    container: &'a str,
    blob_service: &'a State<ClientBuilder>,
    value: Form<BlobUploadForm<'a>>,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        blob_service
            .create_or_update(container.to_string(), value.meta.clone(), value.content)
            .await?,
    )))
}

#[delete("/<container>/<prefix..>")]
async fn delete<'a>(
    container: &'a str,
    prefix: Segments<'a, Path>,
    blob_service: &'a State<ClientBuilder>,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    let prefix = prefix
        .into_iter()
        .fold(String::new(), |f, s| format!("{}/{}", f, s))[1..]
        .to_string();
    let deleted = blob_service
        .delete(container.to_string(), prefix.to_string())
        .await?;
    Ok(Json(ApiResponse::ok(format!("deleted: {} blobs", deleted))))
}
