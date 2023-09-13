use super::{controller::Controller, response::ApiResponse};
use crate::{
    auth::claims::Claims,
    error::{ApiError, Error},
    services::azure_blob::AzureBlobService,
};
use rocket::{
    data::{Limits, ToByteUnit},
    get,
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
        routes![upload]
    }
}

#[put("/", data = "<img>")]
async fn upload<'a>(
    _claims: Claims,
    content_type: &ContentType,
    limits: &Limits,
    blob_service: &State<AzureBlobService>,
    img: Data<'a>,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    if !content_type.is_jpeg() && !content_type.is_png() && !content_type.is_bmp() {
        println!("Invalid media type");
        return Err(Error::from(Status::BadRequest).into());
    }
    let Some(ext) = content_type.extension() else {
        println!("No ext");
        return Err(Error::from(Status::BadRequest).into());
    };
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    let data = img
        .open(limits.get("file").unwrap_or(5.mebibytes()))
        .into_bytes()
        .await
        .map_err(|_| Error::from(Status::InternalServerError))?;
    if !data.is_complete() {
        return Err(Error::from(Status::PayloadTooLarge).into());
    }
    blob_service
        .upload(filename.clone(), data.to_vec(), content_type.to_string())
        .await?;
    Ok(Json(ApiResponse::ok(filename)))
}

#[get("/<key>")]
async fn get(key: String) {}
