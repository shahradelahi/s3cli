use std::time::SystemTime;

use aws_sdk_s3::{Client, Config};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::operation::list_buckets::{ListBucketsError, ListBucketsOutput};
use aws_sdk_s3::operation::list_objects_v2::{ListObjectsV2Error, ListObjectsV2Output};
use aws_sdk_s3::operation::list_objects_v2::builders::ListObjectsV2FluentBuilder;
use colored::Colorize;
use console::Emoji;
use indicatif::{HumanDuration, ProgressBar, ProgressStyle};
use tokio::time::Instant;

use crate::commands::du::DuOpts;
use crate::error::S3Error;
use crate::s3::ParsedS3Url;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

pub struct Bucket {
  pub endpoint: String,
  pub access_key: String,
  pub secret_key: String,
  pub client: Client,
}

fn get_list_object_request(client: &Client, url: &String, delimiter: &char) -> anyhow::Result<ListObjectsV2FluentBuilder> {
  let parsed = parse_url(url, delimiter);
  let mut prefix = parsed.segments.join(delimiter.to_string().as_str());
  if !prefix.is_empty() {
    prefix.push(*delimiter);
  }
  Ok(
    client
       .list_objects_v2()
       .bucket(parsed.bucket_name.to_string())
       .prefix(prefix)
       .max_keys(1000)
  )
}

fn parse_url(url: &String, delimiter: &char) -> ParsedS3Url {
  let parsed = ParsedS3Url::parse_from(url, delimiter).unwrap_or_else(|e| {
    eprintln!("{} {:?}", "error:".red(), e.to_string());
    std::process::exit(1);
  });
  parsed
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

  pub async fn ls(&self, url: &String, delimiter: &char) -> anyhow::Result<Vec<aws_sdk_s3::types::Object>> {
    let mut next_token: Option<String> = None;
    let mut objects: Vec<aws_sdk_s3::types::Object> = Vec::new();
    loop {

      // Get the next page of results
      let response = get_list_object_request(&self.client, url, delimiter)?
         .set_continuation_token(next_token.take())
         .send()
         .await?;

      // Add up the file sizes we got back
      if let Some(contents) = response.contents() {
        for object in contents {
          objects.push(object.clone());
        }
      }

      // Handle pagination, and break the loop if there are no more pages
      next_token = response.continuation_token().map(|t| t.to_string());
      if !response.is_truncated() {
        break;
      }
    }
    Ok(objects)
  }

  pub async fn prefixes(&self, url: &String, delimiter: &char) -> Result<ListObjectsV2Output, S3Error> {
    let request = get_list_object_request(&self.client, url, delimiter)
       .unwrap()
       .delimiter(delimiter.to_string());

    let output = request
       .send()
       .await
       .unwrap_or_else(|e| {
         S3Error::printout_s3sdk_error::<ListObjectsV2Error>(e);
         std::process::exit(1);
       });

    Ok(output)
  }

  /// Lists all objects in an S3 bucket with the given prefix, and adds up their size.
  pub async fn du(&self, opts: DuOpts) -> anyhow::Result<usize> {
    let mut next_token: Option<String> = None;
    let mut total_size_bytes = 0;
    let mut total_objects = 0;
    let started = Instant::now();

    // Print a spinner indicator
    let pb = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
       .unwrap()
       .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    pb.set_message(format!("{} {}", LOOKING_GLASS, "Looking up objects..."));

    if false == *&opts.show_progress {
      pb.finish_and_clear()
    }

    // Loop until we've gotten all the pages of results
    loop {
      // Get the next page of results
      let response = get_list_object_request(&self.client, &opts.prefix, &opts.delimiter)?
         .set_continuation_token(next_token.take())
         .send()
         .await?;

      // Add up the file sizes we got back
      if let Some(contents) = response.contents() {
        for object in contents {
          total_size_bytes += object.size() as usize;
          total_objects += 1;
        }
      }

      if false == pb.is_finished() {
        pb.set_message(format!("{} {}", LOOKING_GLASS, "Looking up objects..."));
      }

      // Handle pagination, and break the loop if there are no more pages
      next_token = response.next_continuation_token().map(|t| t.to_string());
      if !response.is_truncated() || next_token.is_none() {
        break;
      }
    }

    if false == pb.is_finished() {
      pb.finish_and_clear();
      println!("{} {}", "Total Objects:".bold(), total_objects);
      println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
    }

    if *&opts.verbose {
    }

    Ok(total_size_bytes)
  }

  /// Lists all buckets in an S3 account.
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


struct DuOutput {
  pub total_size_bytes: usize,
  pub total_size_human: String,
  pub total_objects: usize,
  pub objects: Vec<aws_sdk_s3::types::Object>,
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

