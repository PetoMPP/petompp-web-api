use azure_storage_blobs::prelude::ClientBuilder;
use petompp_web_models::error::Error;
use rocket::{async_trait, http::Status};

#[async_trait]
pub trait AzureTestService {
    async fn test_connection(&self) -> Result<(), Error>;
}

#[async_trait]
impl AzureTestService for ClientBuilder {
    async fn test_connection(&self) -> Result<(), Error> {
        self.clone()
            .blob_service_client()
            .get_account_information()
            .await
            .map(|_| ())
            .map_err(|e| Error::Status(Status::ServiceUnavailable.code, e.to_string()))
    }
}
