use std::collections::HashMap;
use std::io::Read;
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

use crate::commands::copy::CopyOpts;
use crate::commands::du::DuOpts;
use crate::commands::list::ListOpts;
use crate::error::S3Error;
use crate::s3::content::{S3Directory, S3File};
use crate::s3::ParsedS3Url;

pub mod output;

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

  /// Lists contents of a S3 bucket
  pub async fn ls(&self, opts: ListOpts) -> anyhow::Result<ListOutput> {
    let mut next_token: Option<String> = None;
    let mut objects: Vec<aws_sdk_s3::types::Object> = Vec::new();
    let path = opts.path.unwrap();
    loop {

      // Get the next page of results
      let response = get_list_object_request(&self.client, &path, &opts.delimiter)?
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

    let result = filter_objects_by_path(
      objects,
      &path,
      opts.delimiter,
      opts.recursive,
    );

    Ok(result?)
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

    if *&opts.verbose {}

    Ok(total_size_bytes)
  }

  /// Lists all buckets in an S3 account.
  pub async fn bkt_ls(&self) -> anyhow::Result<ListBucketsOutput> {
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

  /// Copies from content from a bucket to a destination
  pub async fn cp(&self, opts: CopyOpts) -> anyhow::Result<()> {
    Ok(())
  }

  pub async fn mv(&self, from: &String, to: &String) -> Result<(), S3Error> {
    Ok(())
  }

  pub async fn rm(&self, path: &String) -> Result<(), S3Error> {
    Ok(())
  }
}

/// Receives a list of object and a path and returns a list of objects that matches the path
fn filter_objects_by_path(
  objects: Vec<aws_sdk_s3::types::Object>,
  path: &str,
  delimiter: char,
  recursive: bool,
) -> anyhow::Result<ListOutput> {
  let parsed = ParsedS3Url::parse_from(&path.to_string(), &delimiter);
  if let Err(e) = parsed {
    return Err(anyhow::anyhow!("{} {:?}", "error:".red(), e.to_string()));
  };

  let prefix = ensure_trailing_slash(
    &path.to_string()
       .replace("s3://", "")
       .replace(parsed.as_ref().unwrap().bucket_name.as_str(), "")
  );

  let mut output = ListOutput::new();

  for object in objects {
    let mut no_prefixed_key = ensure_surrounding_slashes(object.key.as_ref().unwrap());

    if prefix != "/" {
      no_prefixed_key = no_prefixed_key.replace(&prefix, "");
    }

    let key_segments = no_prefixed_key
       .split(&delimiter.to_string())
       .filter(|&x| !x.is_empty())
       .collect::<Vec<&str>>();

    let first_segment = key_segments.get(0);

    if first_segment.is_some() {
      let key = first_segment.unwrap();
      // if there is not more one segments it means we're same level as the prefix
      // otherwise we're deeper than the prefix, use recursive flag to decide if we should include it
      if key_segments.len() == 1 {
        output.add_object(object.clone());
        continue;
      }

      if false == recursive {
        // it's a directory, in the last statement we collecting objects at same level of the prefix
        let directory = S3Directory::new(key.to_string());
        let directory = output.get_or_insert_directory(directory);
        let file = S3File {
          last_modified: object.last_modified.unwrap(),
          size: object.size.clone(),
          key: object.key.clone().unwrap(),
        };
        directory.add_file(file);
        continue;
      }

      // on recursive mode we're checking if key of object starts with prefix
      if prefix.is_empty() || prefix == "/" {
        output.add_object(object.clone());
      }
    }
  }

  Ok(output)
}

#[derive(Debug)]
pub struct ListOutput {
  pub objects: Vec<aws_sdk_s3::types::Object>,
  pub directories: HashMap<String, S3Directory>,
}

impl ListOutput {
  fn new() -> Self {
    Self {
      objects: Vec::new(),
      directories: HashMap::new(),
    }
  }

  fn add_object(&mut self, object: aws_sdk_s3::types::Object) {
    // first checking if not already exists
    let mut found = false;
    for obj in self.objects.iter_mut() {
      if obj.key == object.key {
        found = true;
        break;
      }
    }
    // otherwise add it
    if !found {
      self.objects.push(object);
    }
  }

  fn get_or_insert_directory(&mut self, directory: S3Directory) -> &mut S3Directory {
    let dir = self.directories.entry(directory.name.clone()).or_insert(directory);
    dir
  }
}

/// Ensures that a path ends with a trailing slash
fn ensure_trailing_slash(path: &String) -> String {
  if !path.ends_with("/") {
    return format!("{}/", path);
  }
  path.to_string()
}

/// Ensure surrounding slashes
fn ensure_surrounding_slashes(path: &String) -> String {
  if !path.starts_with("/") {
    return format!("/{}", path);
  }
  if !path.ends_with("/") {
    return format!("{}/", path);
  }
  path.to_string()
}

#[cfg(test)]
mod s3_tests {
  use super::*;

  fn setup(large: bool) -> Bucket {
    dotenv::dotenv().ok();
    let endpoint = match large {
      true => std::env::var("ENDPOINT_URL"),
      false => std::env::var("SMALL_ENDPOINT_URL")
    }.unwrap();
    let access_key = std::env::var("ACCESS_KEY").unwrap();
    let secret_key = std::env::var("SECRET_KEY").unwrap();
    Bucket::new(endpoint, access_key, secret_key)
  }

  #[tokio::test]
  async fn test_using_client_directly() {
    let bucket = setup(true);

    let buckets = bucket.client
       .list_buckets()
       .send()
       .await;

    println!("{:?}", buckets);
  }

  #[tokio::test]
  async fn test_ls() {
    let bucket = setup(false);

    let opts = ListOpts {
      recursive: true,
      show_progress: false,
      path: Some(String::from("s3://servicelogs/")),
      delimiter: '/',
      exclude: Vec::new(),
      human_readable: true,
      verbose: true,
    };

    let result = bucket.ls(opts.clone()).await;

    assert!(result.is_ok(), "{}", format!("ls failed: {:?}", &result.err()));

    let result = result.unwrap();

    println!("{:?}", result);
  }
}

