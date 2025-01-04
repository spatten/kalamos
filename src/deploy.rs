use std::{collections::HashSet, fs, path::Path};

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
    #[error("no region")]
    NoRegion,
    #[error("read file error: {0:?}")]
    ReadFile(std::io::Error),
    #[error("strip prefix error: {0:?}")]
    StripPrefixError(std::path::StripPrefixError),
    #[error("generate key error: {0:?}")]
    GenerateKey(std::path::PathBuf),
    #[error("no distribution list")]
    NoDistributionList,
    #[error("cloudfront error: {0:?}")]
    CloudfrontError(AwsError),
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
    let region = response.bucket_region().ok_or(Error::NoRegion)?;
    println!("\n\n{:?}", region);

    // Upload the files to the bucket
    upload_site_to_s3(output_dir, bucket, s3_client).await?;
    // Get the distribution for the bucket and invalidate the cache
    let cloudfront_client = aws_sdk_cloudfront::Client::new(&config);
    invalidate_cloudfront_cache(bucket, region, &cloudfront_client).await?;
    Ok(())
}

async fn upload_site_to_s3(
    site_dir: &Path,
    bucket_name: &str,
    s3_client: aws_sdk_s3::Client,
) -> Result<(), Error> {
    let files = WalkDir::new(site_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file());
    let mut uploaded_files = HashSet::new();
    for file in files {
        let file_path = file.path();
        let file_content = fs::read(file_path).map_err(Error::ReadFile)?;
        let key = file_path
            .strip_prefix(site_dir)
            .map_err(Error::StripPrefixError)?
            .to_str()
            .ok_or(Error::GenerateKey(file_path.to_path_buf()))?;
        let mime_type = mime_guess::from_path(key).first_or_text_plain();
        println!(
            "Uploading path: {:?}, key: {}, mime_type: {}",
            file_path,
            key,
            mime_type.essence_str()
        );
        uploaded_files.insert(key.to_string());
        s3_client
            .put_object()
            .bucket(bucket_name)
            .key(key)
            .body(ByteStream::from(file_content))
            .acl(ObjectCannedAcl::PublicRead)
            .content_type(mime_type.essence_str())
            .send()
            .await
            .map_err(|e| Error::S3Error(AwsError::new(e.to_string())))?;
    }

    // Now remove files that should no longer exist in S3
    // These are files that were previously uploaded but are no longer in the local directory
    let files_on_s3_paginator = s3_client
        .list_objects_v2()
        .bucket(bucket_name)
        .into_paginator()
        .send();
    let files_on_s3_iter = files_on_s3_paginator
        .collect::<Result<Vec<_>, _>>()
        .await
        .map_err(|e| Error::S3Error(AwsError::new(e.to_string())))?;
    let files_on_s3 = files_on_s3_iter
        .into_iter()
        .flat_map(|e| {
            e.contents()
                .iter()
                .filter_map(|obj| obj.key().map(|k| k.to_string()))
                .collect::<Vec<_>>()
        })
        .collect::<HashSet<_>>();
    println!("files on s3: {:?}", files_on_s3);
    println!("uploaded files: {:?}", uploaded_files);
    let files_to_remove = files_on_s3.difference(&uploaded_files);
    println!("files to remove: {:?}", files_to_remove);
    for key in files_to_remove {
        s3_client
            .delete_object()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| Error::S3Error(AwsError::new(e.to_string())))?;
    }
    Ok(())
}

async fn invalidate_cloudfront_cache(
    bucket_name: &str,
    region: &str,
    cloudfront_client: &aws_sdk_cloudfront::Client,
) -> Result<(), Error> {
    let response = cloudfront_client.list_distributions().send().await;
    let distributions = response
        .map_err(|e| Error::CloudfrontError(AwsError::new(e.to_string())))?
        .distribution_list
        .ok_or(Error::NoDistributionList)?
        .items
        .ok_or(Error::NoDistributionList)?;
    let website_origin = format!("{}.s3-website-{}.amazonaws.com", &bucket_name, &region);
    let distribution = distributions.iter().find(|d| match d.origins() {
        Some(origins) => origins
            .clone()
            .items()
            .iter()
            .any(|o| o.domain_name == website_origin),
        None => false,
    });
    let distribution_id = distribution.ok_or(Error::NoDistributionList)?.clone().id;
    println!("\n\ndistribution ID: {:?}", distribution_id);
    let invalidation_paths = Paths::builder()
        .items("/*")
        .quantity(1)
        .build()
        .expect("invalidation paths");
    let now = Utc::now();
    let timestamp = format!("{}", now.timestamp_millis());
    let invalidation_batch = InvalidationBatch::builder()
        .paths(invalidation_paths)
        .caller_reference(timestamp)
        .build()
        .expect("invalidation batch");
    let invalidate_request = cloudfront_client
        .create_invalidation()
        .distribution_id(distribution_id)
        .invalidation_batch(invalidation_batch)
        .send()
        .await;
    println!("{:?}", invalidate_request);
    Ok(())
}
