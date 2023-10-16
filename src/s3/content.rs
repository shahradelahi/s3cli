use std::collections::HashMap;
use std::time::SystemTime;

use crate::utc_datetime;

#[derive(Debug)]
pub struct S3Directory {
  pub name: String,
  contents: HashMap<String, S3File>,
}

impl S3Directory {
  pub fn new(name: String) -> Self {
    Self {
      name,
      contents: HashMap::new(),
    }
  }

  pub fn add_file(&mut self, file: S3File) {
    self.contents.insert(file.key.clone(), file);
  }

  pub fn file_exists(&self, key: &String) -> bool {
    self.contents.contains_key(key)
  }

  pub fn size(&self) -> usize {
    let mut size = 0;
    for (_, file) in self.contents.iter() {
      size += file.size as usize;
    }
    size
  }

  pub fn last_modified(&self) -> aws_sdk_s3::primitives::DateTime {
    let mut last_modified = aws_sdk_s3::primitives::DateTime::from(SystemTime::now());
    for (_, file) in self.contents.iter() {
      if file.last_modified > last_modified {
        last_modified = file.last_modified;
      }
    }
    last_modified
  }
}


#[derive(Debug)]
pub struct S3File {
  pub last_modified: aws_sdk_s3::primitives::DateTime,
  pub size: i64,
  pub key: String,
}

impl S3File {
  fn set_newest(&mut self, last_modified: aws_sdk_s3::primitives::DateTime) {
    if last_modified > self.last_modified {
      self.last_modified = last_modified;
    }
  }

  fn utc_modified_at(&self) -> String {
    utc_datetime(self.last_modified)
  }
}