use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config;

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

pub fn deploy(input_dir: &Path, output_dir: &Path, deploy_config: &Option<DeployConfig>) {
    if let Some(deploy_config) = deploy_config {
        match deploy_config.strategy {
            DeployStrategy::S3AndCloudfront => {
                deploy_to_s3_and_cloudfront(input_dir, output_dir, &deploy_config.bucket);
            }
        }
    }
}

pub fn deploy_to_s3_and_cloudfront(input_dir: &Path, output_dir: &Path, bucket: &str) {
    println!("Deploying to S3 and Cloudfront");
    println!("Input directory: {:?}", input_dir);
    println!("Output directory: {:?}", output_dir);
    println!("Bucket name: {:?}", bucket);
}
