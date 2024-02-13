use azure_core::request_options::Metadata;
use azure_storage_blobs::prelude::*;
use petompp_web_models::{
    error::Error,
    models::blob::blob_meta::{BlobMetaData, BlobMetaDto},
};
use rocket::{async_trait, futures::StreamExt, http::Status};
use std::collections::HashMap;

#[async_trait]
pub trait AzureBlobService {
    async fn get(&self, container: String, filename: String) -> Result<BlobMetaData, Error>;
    async fn get_all(
        &self,
        container: String,
        prefix: Option<String>,
    ) -> Result<Vec<BlobMetaData>, Error>;
    async fn create_or_update(
        &self,
        container: String,
        meta: BlobMetaDto,
        data: &[u8],
    ) -> Result<String, Error>;
    async fn delete(&self, container: String, pattern: String) -> Result<usize, Error>;
}

#[async_trait]
impl AzureBlobService for ClientBuilder {
    async fn get(&self, container: String, filename: String) -> Result<BlobMetaData, Error> {
        let mut stream = self
            .clone()
            .blob_service_client()
            .container_client(container.clone())
            .list_blobs()
            .prefix(filename.clone())
            .include_metadata(true)
            .include_tags(true)
            .include_versions(true)
            .into_stream();
        let mut versions = Vec::new();
        while let Some(resp) = stream.next().await {
            for blob in resp?
                .blobs
                .blobs()
                .filter(|b| &b.name == &filename)
                .cloned()
            {
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
        BlobMetaData::try_from(&curr)
    }

    async fn get_all(
        &self,
        container: String,
        prefix: Option<String>,
    ) -> Result<Vec<BlobMetaData>, Error> {
        let mut builder = self
            .clone()
            .blob_service_client()
            .container_client(container.clone())
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
                // println!("resp: {:?}", &blob.name);
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
                // println!("current: {:?}", &curr_blob.name);
                curr_blob
            })
            .filter_map(|blob| {
                BlobMetaData::try_from(&blob)
                    .map_err(|e| {
                        println!("{:?}", &e);
                        e
                    })
                    .ok()
            })
            .collect())
    }

    async fn create_or_update(
        &self,
        container: String,
        meta: BlobMetaDto,
        data: &[u8],
    ) -> Result<String, Error> {
        let filename = match meta.filename.ends_with("/") {
            true => format!(
                "{}{}{}",
                meta.filename,
                uuid::Uuid::new_v4(),
                meta.content_type
                    .split('/')
                    .last()
                    .map(|ext| format!(".{}", ext))
                    .unwrap_or_default()
            ),
            _ => meta.filename.clone(),
        };
        let mut builder = self
            .clone()
            .blob_client(container, filename.clone())
            .put_block_blob(data.to_vec())
            .metadata(Metadata::from(&meta.metadata))
            .tags(Tags::from(meta.tags.clone()))
            .content_type(meta.content_type.clone());
        if let Some(content_language) = meta.content_language {
            builder = builder.content_language(content_language);
        }
        builder.await?;
        Ok(filename)
    }

    async fn delete(&self, container: String, pattern: String) -> Result<usize, Error> {
        Ok(self
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
                        self.clone()
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
}
