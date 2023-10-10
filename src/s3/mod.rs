#![allow(dead_code, unused_variables)]

use regex::Regex;

pub mod bucket;
pub mod credentials;

#[derive(PartialEq, Debug)]
pub struct ListObjectsResult {
  pub objects: Option<Vec<aws_sdk_s3::types::Object>>,
  pub prefixes: Option<Vec<aws_sdk_s3::types::CommonPrefix>>,
  pub continuation_token: Option<String>,
  pub has_more: bool,
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
