use std::io::Write;

use anyhow::{Error, Result};
use colored::Colorize;

pub mod s3easy;
pub mod error;
pub mod validator;

pub struct Credentials {
  pub access_key: String,
  pub secret_key: String,
}

impl Credentials {
  pub async fn from_env() -> Result<Self> {
    let access_key = std::env::var("AWS_ACCESS_KEY_ID")?;
    let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")?;

    Ok(Self {
      access_key,
      secret_key,
    })
  }

  pub fn from_file(profile_name: &str) -> Result<Self> {
    let dir_cert_path = Self::get_certs_directory()?;
    let cert_path = format!("{}/credentials", dir_cert_path);

    if false == check_file_exists(&cert_path) {
      return Err(Error::msg(format!("File {} not found", cert_path.bold())));
    }

    if false == Self::profile_exists(profile_name)? {
      return Err(Error::msg(format!("Profile {} not found", profile_name.bold())));
    }

    let file = std::fs::read_to_string(&cert_path)?;
    let mut lines = file.lines();

    let mut profile_lines = vec![];
    while let Some(line) = lines.next() {
      if line == format!("[{}]", profile_name) {
        while let Some(line) = lines.next() {
          if line == "" || line.starts_with("[") {
            break;
          }

          // Pushing profile lines to a vector
          profile_lines.push(line.to_string());
        }
        break;
      }
    }

    let mut access_key = String::new();
    let mut secret_key = String::new();

    for line in profile_lines {
      let mut parts = line.split("=");
      let key = parts.next().unwrap().trim().to_lowercase();
      let value = parts.next().unwrap().trim().to_string();

      if key == "aws_access_key_id" {
        access_key = value;
      } else if key == "aws_secret_access_key" {
        secret_key = value;
      }
    }

    Ok(Self { access_key, secret_key })
  }

  pub fn get_certs_directory() -> Result<String> {
    let user_home = std::env::var("HOME")?;
    Ok(format!("{}/.aws", user_home))
  }

  pub fn ensure_certs_directory() -> Result<()> {
    let dir_cert_path = Self::get_certs_directory()?;
    if false == check_file_exists(&dir_cert_path) {
      std::fs::create_dir_all(&dir_cert_path)?;
    }
    let cert_path = format!("{}/credentials", dir_cert_path);
    if false == check_file_exists(&cert_path) {
      std::fs::File::create(&cert_path)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests_leading_certs {
  use super::*;

  #[test]
  fn test_check_file_exists() {
    assert_eq!(true, check_file_exists("/etc/passwd"));
    assert_eq!(false, check_file_exists("/etc/passwd123"));
  }

  #[test]
  fn test_credentials_from_file() {
    let credentials = Credentials::from_file("default");
    if let Err(e) = &credentials {
      eprintln!("{e:?}");
      assert!(false);
    }
    let credentials = credentials.unwrap();
    assert_eq!(true, credentials.access_key.len() > 0);
    assert_eq!(true, credentials.secret_key.len() > 0);
  }

  #[test]
  fn test_credentials_from_file_not_found() {
    let credentials = Credentials::from_file("default123");
    if let Err(e) = &credentials {
      assert_eq!(true, e.to_string().contains("Profile default123 not found"));
    } else {
      assert!(false);
    }
  }
}

impl Credentials {
  pub fn profile_save(&self, profile: &str) -> Result<()> {
    if false == validator::check_profile_name(profile) {
      return Err(Error::msg("Profile name is not valid, it should be at least 2 characters long and only contains letters, numbers, - and _"));
    }

    let dir_cert_path = Self::get_certs_directory()?;
    let cert_path = format!("{}/credentials", dir_cert_path);

    Self::ensure_certs_directory()?;

    if false == check_file_exists(&cert_path) {
      return Err(Error::msg(format!("File {} not found", cert_path.bold())));
    }

    if Self::profile_exists(profile)? {
      return Err(Error::msg(format!("Profile {} already exists", profile.bold())));
    }

    let config = vec![
      format!("[{}]", profile),
      format!("aws_access_key_id = {}", self.access_key),
      format!("aws_secret_access_key = {}", self.secret_key),
    ];

    let mut file = std::fs::OpenOptions::new().append(true).open(&cert_path)?;
    file.write_all(format!("\n{}", config.join("\n")).as_bytes())?;

    Ok(())
  }

  pub fn profile_exists(profile_name: &str) -> Result<bool> {
    let dir_cert_path = Self::get_certs_directory()?;
    let cert_path = format!("{}/credentials", dir_cert_path);

    if false == check_file_exists(&cert_path) {
      return Ok(false);
    }

    let file = std::fs::read_to_string(&cert_path)?;
    let mut lines = file.lines();
    let mut found = false;
    while let Some(line) = lines.next() {
      if line == format!("[{}]", profile_name) {
        found = true;
        break;
      }
    }

    Ok(found)
  }

  pub fn profile_remove(profile_name: &str) -> Result<()> {
    let dir_cert_path = Self::get_certs_directory()?;
    let cert_path = format!("{}/credentials", dir_cert_path);

    if false == check_file_exists(&cert_path) {
      return Ok(());
    }

    let file = std::fs::read_to_string(&cert_path)?;
    let mut lines = file.lines();
    let mut new_lines = vec![];
    while let Some(line) = lines.next() {
      if line == format!("[{}]", profile_name) {
        while let Some(line) = lines.next() {
          if line == "" || line.starts_with("[") {
            new_lines.push(line.to_string());
          }
        }
        break;
      }
    }

    // This means there is no other profiles, clean the file
    if new_lines.len() == 0 {
      std::fs::remove_file(&cert_path)?;
      return Ok(());
    }

    let new_file = new_lines.join("\n");
    std::fs::write(&cert_path, new_file)?;

    Ok(())
  }
}

pub fn check_file_exists(path: &str) -> bool {
  std::path::Path::new(path).exists()
}

#[cfg(test)]
mod tests_profile_management {
  use super::*;

  #[test]
  fn test_create_profile() {
    let profile_name = "test-profile";
    let credentials = Credentials {
      access_key: String::from("xxxxxxxxxxxxxxxxxxxxxxxxxxx"),
      secret_key: String::from("xxxxxxxxxxxxxxxxxxxxxxxxxxx"),
    };
    let result = credentials.profile_save(profile_name);
    if let Err(e) = result {
      eprintln!("{e:?}");
      assert!(false);
    }
  }

  #[test]
  fn test_remove_profile() {
    let profile_name = "test-profile";
    let result = Credentials::profile_remove(profile_name);
    if let Err(e) = result {
      eprintln!("{e:?}");
      assert!(false);
    }
  }
}

