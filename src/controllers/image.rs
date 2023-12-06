use super::controller::Controller;
use crate::{
    auth::claims::{AdminClaims, Claims},
    services::azure_blob::AzureBlobService,
};
use petompp_web_models::{
    error::{ApiError, Error},
    models::api_response::ApiResponse,
};
use rocket::{
    data::{Limits, ToByteUnit},
    delete, get,
    http::{ContentType, Status},
    put, routes,
    serde::json::Json,
    Data, State,
};

pub struct ImageController;

impl Controller for ImageController {
    fn path(&self) -> &'static str {
        "/img"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![upload, get_all, delete]
    }
}

#[get("/")]
async fn get_all(
    blob_service: &State<AzureBlobService>,
) -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    Ok(Json(ApiResponse::ok(blob_service.get_image_paths().await?)))
}

#[put("/?<folder>&<filename>", data = "<img>")]
async fn upload<'a>(
    folder: &'a str,
    filename: Option<&'a str>,
    _claims: Claims,
    content_type: &ContentType,
    limits: &Limits,
    blob_service: &State<AzureBlobService>,
    img: Data<'a>,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    if !content_type.is_jpeg() && !content_type.is_png() && !content_type.is_bmp() {
        return Err(Error::from(Status::BadRequest).into());
    }
    let Some(ext) = content_type.extension() else {
        return Err(Error::from(Status::BadRequest).into());
    };
    let filename = match filename {
        Some(filename) => format!("{}.{}", filename, ext),
        None => format!("{}.{}", uuid::Uuid::new_v4(), ext),
    };
    let data = img
        .open(limits.get("file").unwrap_or(20.mebibytes()))
        .into_bytes()
        .await
        .map_err(|_| Error::from(Status::InternalServerError))?;
    if !data.is_complete() {
        return Err(Error::from(Status::PayloadTooLarge).into());
    }
    blob_service
        .upload_img(
            filename.clone(),
            folder.to_string(),
            data.to_vec(),
            content_type.to_string(),
        )
        .await?;
    Ok(Json(ApiResponse::ok(filename)))
}

#[delete("/?<pattern>")]
async fn delete<'a>(
    pattern: &'a str,
    _claims: AdminClaims,
    blob_service: &State<AzureBlobService>,
) -> Result<Json<ApiResponse<'a, usize>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(
        blob_service.delete_img(pattern.to_string()).await?,
    )))
}
