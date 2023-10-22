use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use petompp_web_models::{
    error::Error,
    models::{blog_data::BlogMetaData, country::Country},
};
use rocket::futures::StreamExt;

#[derive(Debug)]
pub struct AzureBlobSecrets {
    pub account: String,
    pub account_key: String,
}

impl Default for AzureBlobSecrets {
    fn default() -> Self {
        Self {
            account: std::env::var("STORAGE_ACCOUNT").expect("STORAGE_ACCOUNT must be set"),
            account_key: std::env::var("STORAGE_ACCESS_KEY")
                .expect("STORAGE_ACCESS_KEY must be set"),
        }
    }
}

pub struct AzureBlobService {
    client: ClientBuilder,
}

impl AzureBlobService {
    pub fn new(secrets: AzureBlobSecrets) -> Self {
        let creds =
            StorageCredentials::access_key(secrets.account.clone(), secrets.account_key.clone());
        let client = ClientBuilder::new(&secrets.account, creds);
        Self { client }
    }

    const IMAGE_CONTAINER: &str = "image-upload";
    pub async fn upload_img(
        &self,
        name: String,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<(), Error> {
        const IMAGE_FOLDER: &str = "editor";
        Ok(self
            .client
            .clone()
            .blob_client(
                Self::IMAGE_CONTAINER.to_string(),
                format!("{}/{}", IMAGE_FOLDER, name),
            )
            .put_block_blob(data)
            .content_type(content_type)
            .await
            .map(|_| ())?)
    }

    const BLOG_CONTAINER: &str = "blog";
    pub async fn get_blog_meta(&self, id: &String, lang: &Country) -> Result<BlogMetaData, Error> {
        let blob_client = &self.client.clone().blob_client(
            Self::BLOG_CONTAINER.to_string(),
            format!("{}/{}.md", id, lang.key()),
        );
        let mut blob = blob_client.get_properties().await?.blob;
        blob.tags = Some(blob_client.get_tags().await?.tags);
        BlogMetaData::try_from(blob)
    }

    pub async fn get_all_blog_meta(&self) -> Result<Vec<BlogMetaData>, Error> {
        let mut stream = self
            .client
            .clone()
            .blob_service_client()
            .container_client(Self::BLOG_CONTAINER.to_string())
            .list_blobs()
            .include_metadata(true)
            .include_tags(true)
            .into_stream();
        let mut result = Vec::new();
        while let Some(resp) = stream.next().await {
            for blob in resp?.blobs.blobs().cloned() {
                result.push(BlogMetaData::try_from(blob)?);
            }
        }
        Ok(result)
    }
}
