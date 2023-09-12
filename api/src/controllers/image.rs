use crate::{
    auth::claims::Claims,
    error::{ApiError, Error},
};

use super::{controller::Controller, response::ApiResponse};
use rocket::{fs::TempFile, get, http::Status, put, routes, serde::json::Json, Build};

pub struct ImageController;

impl Controller for ImageController {
    fn path(&self) -> &'static str {
        "/img"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![upload]
    }

    fn add_managed(&self, rocket: rocket::Rocket<Build>) -> rocket::Rocket<Build> {
        rocket
    }
}

#[put("/", data = "<img>")]
async fn upload<'a>(
    _claims: Claims,
    mut img: TempFile<'a>,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    let Some(content_type) = img.content_type() else {
        println!("No media type");
        return Err(Error::from(Status::BadRequest).into());
    };
    if !content_type.is_jpeg() && !content_type.is_png() && !content_type.is_bmp() {
        println!("Invalid media type");
        return Err(Error::from(Status::BadRequest).into());
    }
    let Some(ext) = content_type.extension() else {
        println!("No ext");
        return Err(Error::from(Status::BadRequest).into());
    };
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    img.persist_to(format!("./temp/{}", &filename))
        .await
        .map_err(|e| {
            println!("{:?}", e);
            Error::from(Status::InternalServerError)
        })?;
    Ok(Json(ApiResponse::ok(filename)))
}

#[get("/<key>")]
async fn get(key: String) {}
