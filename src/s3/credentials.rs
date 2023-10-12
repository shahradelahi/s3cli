pub struct Credentials {
  pub access_key: String,
  pub secret_key: String,
}

impl Credentials {
  pub fn from_env() -> anyhow::Result<Self> {
    let access_key = std::env::var("AWS_ACCESS_KEY_ID")?;
    let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")?;

    Ok(Self {
      access_key,
      secret_key,
    })
  }
}

// #[cfg(test)]
// mod tests_leading_certs {
//   use super::*;
//
//   #[test]
//   fn test_check_file_exists() {
//     assert_eq!(true, std::path::Path::new("/etc/passwd").exists());
//     assert_eq!(false, std::path::Path::new("/etc/passwd123").exists());
//   }
//
//   #[test]
//   fn test_credentials_from_file() {
//     let credentials = Credentials::from_file("default");
//     if let Err(e) = &credentials {
//       eprintln!("{e:?}");
//       assert!(false);
//     }
//     let credentials = credentials.unwrap();
//     assert_eq!(true, credentials.access_key.len() > 0);
//     assert_eq!(true, credentials.secret_key.len() > 0);
//   }
//
//   #[test]
//   fn test_credentials_from_file_not_found() {
//     let credentials = Credentials::from_file("default123");
//     if let Err(e) = &credentials {
//       assert_eq!(true, e.to_string().contains("Profile default123 not found"));
//     } else {
//       assert!(false);
//     }
//   }
// }

