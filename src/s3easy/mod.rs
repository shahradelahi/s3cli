#![allow(dead_code, unused_variables)]

use std::time::SystemTime;

use aws_sdk_s3::{Client, Config};
use aws_sdk_s3::config::Credentials;
use regex::Regex;

use crate::error::S3Error;

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


#[derive(PartialEq, Debug)]
pub struct ListObjectsResult {
  pub objects: Option<Vec<aws_sdk_s3::types::Object>>,
  pub prefixes: Option<Vec<aws_sdk_s3::types::CommonPrefix>>,
  pub continuation_token: Option<String>,
  pub has_more: bool,
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

pub struct ParsedS3Url<'a, 'b> {
  bucket_name: &'a str,
  segments: Vec<&'b str>,
}

impl ParsedS3Url<'_, '_> {
  // regex for s3://<bucket-name>/<folder>/<file>
  fn get_s3url_regex() -> Regex {
    Regex::new(r"^s3://([a-zA-Z0-9\-_]+)/?(.*)$").unwrap()
  }

  pub fn parse_from<'a>(url: &'a str) -> ParsedS3Url<'a, '_> {
    let captured = ParsedS3Url::get_s3url_regex()
       .captures(url)
       .unwrap();

    let bucket_name = captured.get(1).unwrap().as_str();
    let segments = captured.get(2).unwrap().as_str().split("/")
       .filter(|s| !s.is_empty())
       .collect();

    ParsedS3Url {
      bucket_name,
      segments,
    }
  }

  pub fn is_s3url(url: &str) -> bool {
    ParsedS3Url::get_s3url_regex()
       .captures(url)
       .is_some()
  }
}


#[cfg(test)]
mod tests_parsed_3url {
  use super::*;

  #[test]
  fn test_can_parse() {
    let parsed = ParsedS3Url::parse_from("s3://servicelogs/Documents/myfile.txt");
    assert_eq!(parsed.bucket_name, "servicelogs");
    assert_eq!(parsed.segments, vec!["Documents", "myfile.txt"]);

    let parsed = ParsedS3Url::parse_from("s3://servicelogs");
    assert_eq!(parsed.bucket_name, "servicelogs");
    assert_eq!(parsed.segments, vec![] as Vec<String>);
  }

  #[test]
  fn test_can_detect_s3urls() {
    assert!(ParsedS3Url::is_s3url("s3://servicelogs/Documents/myfile.txt"));
    assert!(ParsedS3Url::is_s3url("s3://servicelogs"));
    assert_eq!(false, ParsedS3Url::is_s3url("http://servicelogs/sad"));
  }
}

pub enum ACL {
  Private,
  PublicRead,
  PublicReadWrite,
  AuthenticatedRead,
  AWSExecRead,
  BucketOwnerRead,
  BucketOwnerFC,
}

impl ACL {
  pub fn to_string(&self) -> String {
    match self {
      ACL::Private => String::from("private"),
      ACL::PublicRead => String::from("public-read"),
      ACL::PublicReadWrite => String::from("public-read-write"),
      ACL::AuthenticatedRead => String::from("authenticated-read"),
      ACL::AWSExecRead => String::from("aws-exec-read"),
      ACL::BucketOwnerRead => String::from("bucket-owner-read"),
      ACL::BucketOwnerFC => String::from("bucket-owner-full-control"),
    }
  }
}
