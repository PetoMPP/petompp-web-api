use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::ClientBuilder;

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

impl From<AzureBlobSecrets> for ClientBuilder {
    fn from(secrets: AzureBlobSecrets) -> Self {
        ClientBuilder::new(
            &secrets.account,
            StorageCredentials::access_key(secrets.account.clone(), secrets.account_key.clone()),
        )
    }
}
