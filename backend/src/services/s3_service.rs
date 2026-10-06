//! S3 storage service
//!
//! Provides file upload and delete operations for S3-compatible storage.

use aws_config::BehaviorVersion;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use uuid::Uuid;

use crate::config::S3Config;
use crate::error::ApiError;

/// S3 service for file operations
#[derive(Clone)]
pub struct S3Service {
    client: Client,
    bucket: String,
    endpoint: String,
    public_url: String,
}

/// Result of a file upload operation
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub object_key: String,
    pub url: String,
    pub bucket: String,
}

pub struct DownloadResult {
    pub data: Vec<u8>,
    pub content_type: String,
}

impl S3Service {
    /// Create a new S3 service instance
    pub async fn new(config: &S3Config) -> Result<Self, ApiError> {
        for (name, value) in [
            ("endpoint", config.endpoint.as_str()),
            ("bucket", config.bucket.as_str()),
            ("access_key", config.access_key.as_str()),
            ("secret_key", config.secret_key.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(ApiError::FileUploadError(format!(
                    "S3 configuration is missing {name}"
                )));
            }
        }

        let credentials = Credentials::new(
            &config.access_key,
            &config.secret_key,
            None,
            None,
            "blog-backend",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .endpoint_url(&config.endpoint)
            .credentials_provider(credentials)
            .force_path_style(true) // Required for MinIO and other S3-compatible services
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: config.bucket.clone(),
            endpoint: config.endpoint.clone(),
            public_url: config.public_url.clone(),
        })
    }

    /// Upload a file to S3
    ///
    /// # Arguments
    /// * `data` - File content as bytes
    /// * `original_filename` - Original filename for extension extraction
    /// * `content_type` - MIME type of the file
    ///
    /// # Returns
    /// Upload result containing the object key and URL
    pub async fn upload_file(
        &self,
        data: Vec<u8>,
        original_filename: &str,
        content_type: &str,
    ) -> Result<UploadResult, ApiError> {
        // Generate unique object key with original extension
        let extension = std::path::Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let object_key = if extension.is_empty() {
            format!("uploads/{}", Uuid::new_v4())
        } else {
            format!("uploads/{}.{}", Uuid::new_v4(), extension)
        };

        self.ensure_bucket().await?;

        // Upload to S3
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&object_key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("S3 upload error: {:?}", e);
                ApiError::FileUploadError(format!("Failed to upload file: {}", e))
            })?;

        // Prefer configured public URL for browser access. Fall back to endpoint-style URLs.
        let url = if self.public_url.trim().is_empty() {
            format!(
                "{}/{}/{}",
                self.endpoint.trim_end_matches('/'),
                self.bucket,
                object_key
            )
        } else {
            format!("{}/{}", self.public_url.trim_end_matches('/'), object_key)
        };

        tracing::info!("File uploaded successfully: {}", object_key);

        Ok(UploadResult {
            object_key,
            url,
            bucket: self.bucket.clone(),
        })
    }

    /// Download an object for public delivery through the application API.
    pub async fn download_file(&self, object_key: &str) -> Result<DownloadResult, ApiError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("S3 download error: {:?}", e);
                ApiError::NotFound("File content is unavailable".to_string())
            })?;

        let content_type = output
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let data = output.body.collect().await.map_err(|e| {
            tracing::error!("S3 response body error: {:?}", e);
            ApiError::FileUploadError("Failed to read stored file".to_string())
        })?;

        Ok(DownloadResult {
            data: data.into_bytes().to_vec(),
            content_type,
        })
    }

    async fn ensure_bucket(&self) -> Result<(), ApiError> {
        if self
            .client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .is_ok()
        {
            return Ok(());
        }

        if let Err(create_error) = self
            .client
            .create_bucket()
            .bucket(&self.bucket)
            .send()
            .await
        {
            // Another request may have created the bucket between the first
            // HEAD request and CREATE. Treat that race as success.
            if self
                .client
                .head_bucket()
                .bucket(&self.bucket)
                .send()
                .await
                .is_ok()
            {
                return Ok(());
            }

            tracing::error!("S3 bucket initialization error: {:?}", create_error);
            return Err(ApiError::FileUploadError(format!(
                "Failed to initialize storage bucket '{}': {}",
                self.bucket, create_error
            )));
        }

        tracing::info!("S3 bucket initialized: {}", self.bucket);
        Ok(())
    }

    /// Delete a file from S3
    ///
    /// # Arguments
    /// * `object_key` - The S3 object key to delete
    pub async fn delete_file(&self, object_key: &str) -> Result<(), ApiError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(object_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("S3 delete error: {:?}", e);
                ApiError::FileUploadError(format!("Failed to delete file: {}", e))
            })?;

        tracing::info!("File deleted successfully: {}", object_key);

        Ok(())
    }
}
