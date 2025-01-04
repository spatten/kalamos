use std::{fs, path::Path};

use aws_sdk_cloudfront::types::{InvalidationBatch, Paths};
use aws_sdk_s3::{primitives::ByteStream, types::ObjectCannedAcl};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use crate::{config, render};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStrategy {
    #[serde(rename = "s3_and_cloudfront")]
    S3AndCloudfront,
}

impl From<config::DeployStrategy> for DeployStrategy {
    fn from(strategy: config::DeployStrategy) -> Self {
        match strategy {
            config::DeployStrategy::S3AndCloudfront => DeployStrategy::S3AndCloudfront,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub strategy: DeployStrategy,
    pub bucket: String,
}

impl From<config::DeployConfig> for DeployConfig {
    fn from(config: config::DeployConfig) -> Self {
        Self {
            strategy: config.strategy.into(),
            bucket: config.bucket,
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("render error: {0}")]
    RenderError(render::Error),
    #[error("s3 error: {0:?}")]
    S3Error(AwsError),
}

#[derive(Debug)]
pub struct AwsError(String);
impl AwsError {
    pub fn new(value: impl Into<String>) -> Self {
        AwsError(value.into())
    }

    pub fn add_message(self, message: impl Into<String>) -> Self {
        AwsError(format!("{}: {}", message.into(), self.0))
    }
}

pub async fn deploy(
    input_dir: &Path,
    output_dir: &Path,
    deploy_config: &Option<DeployConfig>,
) -> Result<(), Error> {
    if let Some(deploy_config) = deploy_config {
        match deploy_config.strategy {
            DeployStrategy::S3AndCloudfront => {
                deploy_to_s3_and_cloudfront(input_dir, output_dir, &deploy_config.bucket).await?;
            }
        }
    }
    Ok(())
}

pub async fn deploy_to_s3_and_cloudfront(
    input_dir: &Path,
    output_dir: &Path,
    bucket: &str,
) -> Result<(), Error> {
    println!("Deploying to S3 and Cloudfront");
    println!("Input directory: {:?}", input_dir);
    println!("Output directory: {:?}", output_dir);
    println!("Bucket name: {:?}", bucket);

    render::render_dir(input_dir, output_dir).map_err(Error::RenderError)?;
    let config = aws_config::from_env().load().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let response = s3_client
        .head_bucket()
        .bucket(bucket)
        .send()
        .await
        .map_err(|e| Error::S3Error(AwsError::new(e.to_string())))?;
    let region = response.bucket_region().unwrap();
    println!("\n\n{:?}", region);

    // Upload the files to the bucket
    upload_site_to_s3(output_dir, bucket, s3_client).await;
    // Get the distribution for the bucket and invalidate the cache
    let cloudfront_client = aws_sdk_cloudfront::Client::new(&config);
    invalidate_cloudfront_cache(bucket, region, &cloudfront_client).await;
    Ok(())
}

async fn upload_site_to_s3(site_dir: &Path, bucket_name: &str, s3_client: aws_sdk_s3::Client) {
    let files = WalkDir::new(site_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file());
    for file in files {
        let file_path = file.path();
        let file_content = fs::read(file_path).unwrap();
        let key = file_path.strip_prefix(site_dir).unwrap().to_str().unwrap();
        let mime_type = mime_guess::from_path(key).first_or_text_plain();
        println!(
            "Uploading path: {:?}, key: {}, mime_type: {}",
            file_path,
            key,
            mime_type.essence_str()
        );
        s3_client
            .put_object()
            .bucket(bucket_name)
            .key(key)
            .body(ByteStream::from(file_content))
            .acl(ObjectCannedAcl::PublicRead)
            .content_type(mime_type.essence_str())
            .send()
            .await
            .unwrap();
    }
}

async fn invalidate_cloudfront_cache(
    bucket_name: &str,
    region: &str,
    cloudfront_client: &aws_sdk_cloudfront::Client,
) {
    let response = cloudfront_client.list_distributions().send().await;
    let distributions = response.unwrap().distribution_list.unwrap().items.unwrap();
    let website_origin = format!("{}.s3-website-{}.amazonaws.com", &bucket_name, &region);
    let distribution = distributions.iter().find(|d| match d.origins() {
        Some(origins) => origins
            .clone()
            .items()
            .iter()
            .any(|o| o.domain_name == website_origin),
        None => false,
    });
    let distribution_id = distribution.map(|d| d.clone().id);
    println!("\n\ndistribution ID: {:?}", distribution_id);
    let invalidation_paths = Paths::builder().items("/*").quantity(1).build().unwrap();
    let now = Utc::now();
    let timestamp = format!("{}", now.timestamp_millis());
    let invalidation_batch = InvalidationBatch::builder()
        .paths(invalidation_paths)
        .caller_reference(timestamp)
        .build()
        .unwrap();
    let invalidate_request = cloudfront_client
        .create_invalidation()
        .distribution_id(distribution_id.unwrap())
        .invalidation_batch(invalidation_batch)
        .send()
        .await;
    println!("{:?}", invalidate_request);
}
