use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use petompp_web_models::error::Error;

#[derive(Debug)]
pub struct AzureBlobSecrets {
    pub account: String,
    pub account_key: String,
    pub container_name: String,
}

impl Default for AzureBlobSecrets {
    fn default() -> Self {
        Self {
            account: std::env::var("STORAGE_ACCOUNT").expect("STORAGE_ACCOUNT must be set"),
            account_key: std::env::var("STORAGE_ACCESS_KEY")
                .expect("STORAGE_ACCESS_KEY must be set"),
            container_name: std::env::var("STORAGE_CONTAINER")
                .expect("STORAGE_CONTAINER must be set"),
        }
    }
}

pub struct AzureBlobService {
    secrets: AzureBlobSecrets,
    client: ClientBuilder,
}

impl AzureBlobService {
    pub fn new(secrets: AzureBlobSecrets) -> Self {
        let creds =
            StorageCredentials::access_key(secrets.account.clone(), secrets.account_key.clone());
        let client = ClientBuilder::new(&secrets.account, creds);
        Self { secrets, client }
    }

    pub async fn upload_img(
        &self,
        name: String,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<(), Error> {
        const IMAGE_FOLDER: &str = "image-upload";
        Ok(self
            .client
            .clone()
            .blob_client(
                self.secrets.container_name.clone(),
                format!("{}/{}", IMAGE_FOLDER, name),
            )
            .put_block_blob(data)
            .content_type(content_type)
            .await
            .map(|_| ())?)
    }
}
