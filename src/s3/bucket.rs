use std::time::SystemTime;

use aws_sdk_s3::{Client, Config};
use aws_sdk_s3::config::Credentials;

use crate::error::S3Error;
use crate::s3::{ListObjectsResult, ParsedS3Url};

pub struct Bucket {
  pub endpoint: String,
  pub access_key: String,
  pub secret_key: String,
  pub client: Client,
}

impl Bucket {
  pub async fn new(endpoint: String, access_key: String, secret_key: String) -> Self {
    let client_config = Config::builder()
       .force_path_style(true)
       .credentials_provider(
         Credentials::new(
           access_key.to_string(),
           secret_key.to_string(),
           Some(String::from("")),
           Some(SystemTime::now()),
           "s3cli-unknown-bucket",
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

  pub async fn ls(&self, url: &str) -> Result<ListObjectsResult, S3Error> {
    let parsed = ParsedS3Url::parse_from(url);

    let request = self.client
       .list_objects_v2()
       .bucket(parsed.bucket_name.to_string())
       // We want to get contents of current directory
       .prefix(parsed.segments.join("/") + "/")
       .delimiter("/");

    println!("Bucket: {:?}", &request.get_bucket());
    println!("Prefix: {:?}", &request.get_prefix());
    println!("Delimiter: {:?}", &request.get_delimiter());

    let output = request
       .send()
       .await
       .unwrap_or_else(|e| panic!("Error: {:?}", e));

    Ok(ListObjectsResult {
      objects: output.contents().map(|o| o.to_vec()),
      prefixes: output.common_prefixes().map(|p| p.to_vec()),
      continuation_token: output.continuation_token().map(|t| t.to_string()),
      has_more: output.is_truncated(),
    })
  }

  pub async fn mv(&self, from: String, to: String) -> Result<(), S3Error> {
    Ok(())
  }

  pub async fn rm(&self, path: String) -> Result<(), S3Error> {
    Ok(())
  }

  pub async fn cp(&self, from: String, to: String) -> Result<(), S3Error> {
    // 1. One of (from, to) must be S3Url
    let least_one_s3 = vec![from, to].iter().map(|path| {});

    // 2.

    Ok(())
  }
}

#[cfg(test)]
mod s3_tests {
  use super::*;

  async fn setup() -> Bucket {
    Bucket::new(
      "https://s3.ir-thr-at1.arvanstorage.ir".to_string(),
      "6803f5a1-eb55-4812-b375-28b0eca6c70b".to_string(),
      "2d8c546e7e1962d9cdd5c829898979eff68f1562".to_string(),
    ).await
  }

  #[tokio::test]
  async fn test_ls() {
    let bucket = setup().await;

    let result = bucket
       .ls("s3://staticresources/")
       .await
       .unwrap();

    println!("{:?}", result);
  }

  #[tokio::test]
  async fn test_using_client_directly() {
    let bucket = setup().await;

    let buckets = bucket.client
       .list_buckets()
       .send()
       .await;

    println!("{:?}", buckets);
  }
}
