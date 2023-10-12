#![allow(dead_code, unused_variables)]

use regex::Regex;

pub mod bucket;
pub mod credentials;
pub mod profile;

#[derive(PartialEq, Debug)]
pub struct ListObjectsResult {
  pub objects: Option<Vec<aws_sdk_s3::types::Object>>,
  pub common_prefixes: Option<Vec<aws_sdk_s3::types::CommonPrefix>>,
  // pub continuation_token: Option<String>,
  pub has_more: bool,
}

pub struct ParsedS3Url {
  bucket_name: String,
  segments: Vec<String>,
}

impl ParsedS3Url {
  // regex for s3://<bucket-name>/<folder>/<file>
  fn get_s3url_regex() -> Regex {
    // Regex::new(r"^s3://([a-zA-Z0-9\-_]+)/?(.*)$").unwrap()
    Regex::new(r"^s3://([0-9a-zA-Z!\-_.*'()]+/?)*").unwrap()
  }

  pub fn parse_from(url: &String, delimiter: &char) -> anyhow::Result<ParsedS3Url> {
    if false == Self::is_s3url(url.as_str()) {
      return Err(anyhow::Error::msg("Not a valid S3 URL"));
    }

    let binding = url.to_owned().replace("s3://", "");

    let parts = binding
       .split(*delimiter)
       .collect::<Vec<&str>>();

    let bucket_name = parts.get(0);
    if bucket_name.is_none() || bucket_name.unwrap().is_empty() {
      return Err(anyhow::Error::msg("Bucket name is required"));
    }

    let segments = parts[1..]
       .iter()
       .filter(|s| !s.is_empty())
       .map(|i| i.to_string())
       .collect();

    Ok(ParsedS3Url {
      bucket_name: bucket_name.unwrap().to_string(),
      segments,
    })
  }

  // https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html
  // The following character sets are generally safe for use in key names.
  //
  // Alphanumeric characters
  // - 0-9
  // - a-z
  // - A-Z
  //
  // Special characters
  // - Exclamation point (!)
  // - Hyphen (-)
  // - Underscore (_)
  // - Period (.)
  // - Asterisk (*)
  // - Single quote (')
  // - Open parenthesis (()
  // - Close parenthesis ())
  pub fn is_s3url(url: &str) -> bool {
    url.starts_with("s3://") && Self::get_s3url_regex().is_match(url)
  }
}



#[cfg(test)]
mod tests_parsed_3url {
  use super::*;

  #[test]
  fn test_can_parse() {
    let parsed = ParsedS3Url::parse_from(&String::from("s3://servicelogs/Documents/myfile.txt"), &'/')
       .expect("failed to parse s3url");
    assert_eq!(parsed.bucket_name, "servicelogs");
    assert_eq!(parsed.segments, vec!["Documents", "myfile.txt"]);

    let parsed = ParsedS3Url::parse_from(&String::from("s3://servicelogs"), &'/')
       .expect("failed to parse s3url");
    assert_eq!(parsed.bucket_name, "servicelogs");
    assert_eq!(parsed.segments, vec![] as Vec<String>);

    let parsed = ParsedS3Url::parse_from(&String::from("s3://my.great_photos-2014/jan/myvacation.jpg"), &'/')
       .expect("failed to parse s3url");
    assert_eq!(parsed.bucket_name, "my.great_photos-2014");
    assert_eq!(parsed.segments, vec!["jan", "myvacation.jpg"]);
  }

  #[test]
  fn test_captures() {
    let captures = ParsedS3Url::get_s3url_regex()
       .captures("s3://servicelogs/Documents/myfile.txt")
       .unwrap();

    for capture in captures.iter() {
      println!("{:?}", capture.unwrap());
    }
  }

  #[test]
  fn test_can_detect_s3urls() {
    assert!(ParsedS3Url::is_s3url("s3://")); // Refers to the root of a bucket
    assert!(ParsedS3Url::is_s3url("s3://4my-organization"));
    assert!(ParsedS3Url::is_s3url("s3://my.great_photos-2014/jan/myvacation.jpg"));
    assert!(ParsedS3Url::is_s3url("s3://videos/2014/birthday/video1.wmv"));
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
