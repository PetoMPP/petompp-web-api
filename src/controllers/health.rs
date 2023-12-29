use super::controller::Controller;
use crate::{services::azure_blob::AzureBlobService, PgPool};
use diesel::r2d2::R2D2Connection;
use petompp_web_models::{
    error::{ApiError, Error},
    models::api_response::ApiResponse,
};
use rocket::{get, routes, serde::json::Json};

pub struct HealthController;

impl Controller for HealthController {
    fn path(&self) -> &'static str {
        "/health"
    }

    fn routes(&self) -> Vec<rocket::Route> {
        routes![health]
    }
}

#[get("/")]
async fn health<'a>(
    pool: &'a rocket::State<PgPool>,
    azure_blob_service: &'a rocket::State<AzureBlobService>,
) -> Result<Json<ApiResponse<'a, &'a str>>, ApiError<'a>> {
    // test database connection
    let mut conn = pool.get().map_err::<Error, _>(|e| e.into())?;
    conn.ping().map_err::<Error, _>(|e| e.into())?;

    // test sevices connection
    azure_blob_service.test_connection().await?;

    Ok(Json(ApiResponse::ok("ok")))
}
