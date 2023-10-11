use std::error;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;

use aws_sdk_s3::error::SdkError;
use colored::Colorize;

#[derive(Debug)]
pub struct S3Error;

impl S3Error {
  pub fn printout_s3sdk_error<E: fmt::Debug>(err: SdkError<E>) {
    match err {
      SdkError::DispatchFailure(e) => {
        let cne = e.as_connector_error().unwrap();
        eprintln!("{} {}: {:?}", "error:".red(), &cne.to_string(), &cne.source().unwrap().to_string());
      }
      e => eprintln!("{} {:?}", "error:".red(), e)
    }
  }
}

impl fmt::Display for S3Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "S3Error")
  }
}

impl error::Error for S3Error {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}



