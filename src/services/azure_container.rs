use azure_core::StatusCode;
use azure_storage_blobs::{container::PublicAccess, prelude::ClientBuilder};
use petompp_web_models::error::Error;
use rocket::{async_trait, futures::StreamExt};

#[async_trait]
pub trait AzureContainerService {
    async fn get(&self, name: &str) -> Result<String, Error>;
    async fn get_all(&self) -> Result<Vec<String>, Error>;
    async fn create(&self, name: &str) -> Result<(), Error>;
}

#[async_trait]
impl AzureContainerService for ClientBuilder {
    async fn get(&self, name: &str) -> Result<String, Error> {
        let client = self.clone().blob_service_client().container_client(name);
        match client
            .exists()
            .await
            .map_err(|e| Error::Database(e.to_string()))?
        {
            true => match client.get_properties().await {
                Ok(resp) => match resp.container.public_access {
                    PublicAccess::Blob => Ok(resp.container.name),
                    _ => Err(Error::Status(404, "Not Found".to_string())),
                },
                Err(e) => Err(e.into()),
            },
            false => Err(Error::Status(404, "Not Found".to_string())),
        }
    }

    async fn get_all(&self) -> Result<Vec<String>, Error> {
        let mut stream = self
            .clone()
            .blob_service_client()
            .list_containers()
            .into_stream();
        let mut containers = Vec::new();
        while let Some(resp) = stream.next().await {
            for container in resp?.containers {
                containers.push(container.name);
            }
        }
        Ok(containers)
    }

    async fn create(&self, name: &str) -> Result<(), Error> {
        let client = self.clone().blob_service_client().container_client(name);
        match client.exists().await? {
            true => Err(Error::Status(
                StatusCode::Conflict.into(),
                StatusCode::Conflict.to_string(),
            )),
            false => match client.create().await {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Database(e.to_string())),
            },
        }
    }
}
