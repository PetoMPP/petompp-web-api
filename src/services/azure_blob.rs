use azure_core::request_options::Metadata;
use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use petompp_web_models::{
    error::Error,
    models::{
        blog_data::{BlogData, BlogMetaData},
        country::Country,
    },
};
use rocket::{futures::StreamExt, http::Status};
use std::collections::HashMap;

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
    pub async fn get_image_paths(&self) -> Result<Vec<String>, Error> {
        let mut stream = self
            .client
            .clone()
            .blob_service_client()
            .container_client(Self::IMAGE_CONTAINER.to_string())
            .list_blobs()
            .into_stream();
        let mut result = Vec::new();
        while let Some(resp) = stream.next().await {
            for blob in resp?.blobs.blobs().cloned() {
                if blob.properties.content_type.starts_with("image/") {
                    result.push(blob.name);
                }
            }
        }
        Ok(result)
    }

    pub async fn upload_img(
        &self,
        name: String,
        folder: String,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<(), Error> {
        Ok(self
            .client
            .clone()
            .blob_client(
                Self::IMAGE_CONTAINER.to_string(),
                format!("{}/{}", folder, name),
            )
            .put_block_blob(data)
            .content_type(content_type)
            .await
            .map(|_| ())?)
    }

    async fn delete(&self, container: String, pattern: String) -> Result<usize, Error> {
        Ok(self
            .client
            .clone()
            .blob_service_client()
            .container_client(container.clone())
            .list_blobs()
            .prefix(pattern)
            .into_stream()
            .fold(Result::<_, Error>::Ok(0), |acc, resp| {
                let container = container.clone();
                async move {
                    let mut count = acc?;
                    for blob in resp?.blobs.blobs().cloned() {
                        self.client
                            .clone()
                            .blob_client(container.clone(), blob.name)
                            .delete()
                            .await?;
                        count += 1;
                    }
                    Ok(count)
                }
            })
            .await?)
    }

    pub async fn delete_img(&self, pattern: String) -> Result<usize, Error> {
        Self::delete(self, Self::IMAGE_CONTAINER.to_string(), pattern).await
    }

    pub async fn delete_blog_post(
        &self,
        id: &String,
        lang: &Country,
    ) -> Result<usize, Error> {
        Self::delete(
            self,
            Self::BLOG_CONTAINER.to_string(),
            format!("{}/{}.md", id, lang.key()),
        )
        .await
    }

    const BLOG_CONTAINER: &str = "blog";
    pub async fn get_blog_meta(&self, id: &String, lang: &Country) -> Result<BlogMetaData, Error> {
        let mut stream = self
            .client
            .clone()
            .blob_service_client()
            .container_client(Self::BLOG_CONTAINER.to_string())
            .list_blobs()
            .prefix(format!("{}/{}.md", id, lang.key()))
            .include_metadata(true)
            .include_tags(true)
            .include_versions(true)
            .into_stream();
        let mut versions = Vec::new();
        while let Some(resp) = stream.next().await {
            for blob in resp?.blobs.blobs().cloned() {
                versions.push(blob);
            }
        }
        let mut curr = versions
            .iter()
            .find(|b| b.is_current_version.unwrap_or_default())
            .cloned()
            .ok_or_else(|| Error::Status(Status::NotFound.code, Status::NotFound.to_string()))?;
        curr.properties.creation_time = versions
            .into_iter()
            .min_by(|x, y| x.properties.creation_time.cmp(&y.properties.creation_time))
            .map(|b| b.properties.creation_time)
            .unwrap();
        BlogMetaData::try_from(curr)
    }

    pub async fn get_all_blog_meta(
        &self,
        prefix: Option<String>,
    ) -> Result<Vec<BlogMetaData>, Error> {
        let mut builder = self
            .client
            .clone()
            .blob_service_client()
            .container_client(Self::BLOG_CONTAINER.to_string())
            .list_blobs()
            .include_metadata(true)
            .include_tags(true)
            .include_versions(true);
        if let Some(prefix) = prefix {
            builder = builder.prefix(prefix);
        }
        let mut stream = builder.into_stream();
        let mut result = HashMap::new();
        while let Some(resp) = stream.next().await {
            for blob in resp?.blobs.blobs().cloned() {
                result
                    .entry(blob.name.clone())
                    .or_insert(vec![blob.clone()])
                    .push(blob);
            }
        }
        if result.is_empty() {
            return Err(Error::Status(
                Status::NotFound.code,
                Status::NotFound.to_string(),
            ));
        }
        Ok(result
            .into_values()
            .filter(|v| v.iter().any(|b| b.is_current_version.unwrap_or_default()))
            .map(|v| {
                let mut curr_blob = v
                    .iter()
                    .find(|b| b.is_current_version.unwrap_or_default())
                    .unwrap()
                    .clone();
                curr_blob.properties.creation_time = v
                    .into_iter()
                    .min_by(|x, y| x.properties.creation_time.cmp(&y.properties.creation_time))
                    .map(|b| b.properties.creation_time)
                    .unwrap();
                curr_blob
            })
            .filter_map(|blob| BlogMetaData::try_from(blob).ok())
            .collect())
    }

    pub async fn create_or_update_blog_post(
        &self,
        id: &String,
        lang: &Country,
        value: &BlogData,
    ) -> Result<(), Error> {
        let lang = lang.key().to_string();
        let meta: Metadata = value.meta.clone().into();
        let tags: Tags = value.meta.tags.clone().into();
        Ok(self
            .client
            .clone()
            .blob_client(
                Self::BLOG_CONTAINER.to_string(),
                format!("{}/{}.md", id, &lang),
            )
            .put_block_blob(value.content.as_bytes().to_vec())
            .metadata(meta)
            .tags(tags)
            .content_type("text/markdown; charset=utf-8")
            .content_language(lang)
            .await
            .map(|_| ())?)
    }
}
