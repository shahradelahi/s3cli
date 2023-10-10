use std::error;
use std::fmt;

#[derive(Debug)]
pub struct S3Error;

impl std::fmt::Display for S3Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
    write!(f, "S3Error")
  }
}

impl error::Error for S3Error {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

pub type HttpResponse = http::Response<aws_sdk_s3::primitives::SdkBody>;
