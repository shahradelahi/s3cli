use std::time::SystemTime;

use aws_sdk_s3::{Client, Config};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::operation::list_buckets::{ListBucketsError, ListBucketsOutput};
use aws_sdk_s3::operation::list_objects::{ListObjectsError, ListObjectsOutput};
use aws_sdk_s3::operation::list_objects::builders::ListObjectsFluentBuilder;
use colored::Colorize;

use crate::error::S3Error;
use crate::s3::ParsedS3Url;

pub struct Bucket {
  pub endpoint: String,
  pub access_key: String,
  pub secret_key: String,
  pub client: Client,
}

impl Bucket {
  pub fn new(endpoint: String, access_key: String, secret_key: String) -> Self {
    let client_config = Config::builder()
       .force_path_style(true)
       .credentials_provider(
         Credentials::new(
           access_key.to_string(),
           secret_key.to_string(),
           Some(String::from("")),
           Some(SystemTime::now()),
           "s3cli",
         )
       )
       .endpoint_url(endpoint.to_string())
       .region(aws_sdk_s3::config::Region::new("us-east-1"))
       .build();

    let client = Client::from_conf(client_config);

    Self {
      endpoint,
      access_key,
      secret_key,
      client,
    }
  }

  fn get_list_object_request(client: &Client, url: &String, delimiter: &char) -> anyhow::Result<ListObjectsFluentBuilder> {
    let parsed = ParsedS3Url::parse_from(url, delimiter).unwrap_or_else(|e| {
      eprintln!("{} {:?}", "error:".red(), e.to_string());
      std::process::exit(1);
    });

    Ok(
      client
         .list_objects()
         .bucket(parsed.bucket_name.to_string())
         .prefix(parsed.segments.join(delimiter.to_string().as_str()) + delimiter.to_string().as_str())
    )
  }

  pub async fn ls(&self, url: &String, delimiter: &char) -> Result<ListObjectsOutput, S3Error> {
    let request = Self::get_list_object_request(&self.client, url, delimiter)
       .unwrap();

    let output = request
       .send()
       .await
       .unwrap_or_else(|e| {
         S3Error::printout_s3sdk_error::<ListObjectsError>(e);
         std::process::exit(1);
       });

    Ok(output)
  }

  pub async fn prefixes(&self, url: &String, delimiter: &char) -> Result<ListObjectsOutput, S3Error> {
    let request = Self::get_list_object_request(&self.client, url, delimiter)
       .unwrap()
       .delimiter(delimiter.to_string());

    let output = request
       .send()
       .await
       .unwrap_or_else(|e| {
         S3Error::printout_s3sdk_error::<ListObjectsError>(e);
         std::process::exit(1);
       });

    Ok(output)
  }

  pub async fn du(&self, url: &String, delimiter: &char) -> Result<(), S3Error> {
    Ok(())
  }

  // This function is for Listing buckets
  pub async fn bkt_ls(&self) -> Result<ListBucketsOutput, S3Error> {
    let request = self.client
       .list_buckets();

    let output = request
       .send()
       .await
       .unwrap_or_else(|e| {
         S3Error::printout_s3sdk_error::<ListBucketsError>(e);
         std::process::exit(1);
       });

    Ok(output)
  }

  pub async fn mv(&self, from: &String, to: &String) -> Result<(), S3Error> {
    Ok(())
  }

  pub async fn rm(&self, path: &String) -> Result<(), S3Error> {
    Ok(())
  }

  pub async fn cp(&self, from: &String, to: &String) -> Result<(), S3Error> {
    // 1. One of (from, to) must be S3Url
    let least_one_s3 = vec![from, to].iter().map(|path| {});

    // 2.

    Ok(())
  }
}

#[cfg(test)]
mod s3_tests {
  use super::*;

  fn setup() -> Bucket {
    dotenv::dotenv().ok();
    let endpoint = std::env::var("ENDPOINT_URL").unwrap();
    let access_key = std::env::var("ACCESS_KEY").unwrap();
    let secret_key = std::env::var("SECRET_KEY").unwrap();
    Bucket::new(endpoint, access_key, secret_key)
  }

  #[tokio::test]
  async fn test_ls() {
    let bucket = setup();

    let result = bucket
       .ls(&String::from("s3://staticresources/"), &'/')
       .await
       .unwrap();

    println!("{:?}", result);
  }

  #[tokio::test]
  async fn test_using_client_directly() {
    let bucket = setup();

    let buckets = bucket.client
       .list_buckets()
       .send()
       .await;

    println!("{:?}", buckets);
  }
}

