use std::collections::HashMap;
use std::io::Write;

use colored::Colorize;

use crate::s3::credentials::Credentials;
use crate::utils::validator;

/// Return a absolute path to `~/.aws` directory
pub fn creds_directory() -> anyhow::Result<String> {
  let user_home = std::env::var("HOME")?;
  Ok(format!("{}/.aws", user_home))
}

/// This function ensures that `~/.aws` directory and `credentials` are already created
pub fn ensure_creds_directory() -> anyhow::Result<()> {
  let dir_cert_path = creds_directory()?;
  if false == std::path::Path::new(&dir_cert_path).exists() {
    std::fs::create_dir_all(&dir_cert_path)?;
  }
  let cert_path = format!("{}/credentials", dir_cert_path);
  if false == std::path::Path::new(&cert_path).exists() {
    std::fs::File::create(&cert_path)?;
  }
  Ok(())
}

/// Characters considered to be whitespace by the spec
///
/// Profile parsing is actually quite strict about what is and is not whitespace, so use this instead
/// of `.is_whitespace()` / `.trim()`
pub(super) const WHITESPACE: &[char] = &[' ', '\t'];
const COMMENT: &[char] = &['#', ';'];

fn is_empty_line(line: &str) -> bool {
  line.trim_matches(WHITESPACE).is_empty()
}

fn is_comment_line(line: &str) -> bool {
  line.starts_with(COMMENT)
}

#[derive(Debug)]
pub struct ProfileSet {
  profiles: HashMap<String, Profile>,
}

impl ProfileSet {
  /// Create a new empty profile set
  pub fn new() -> Self {
    Self { profiles: HashMap::new() }
  }

  /// Loading profiles from ~/.aws/credentials file
  pub fn from_file() -> anyhow::Result<Self> {
    let dir_cert_path = creds_directory()?;
    let creds_path = format!("{}/credentials", dir_cert_path);

    if false == std::path::Path::new(&creds_path).exists() {
      return Err(anyhow::Error::msg(format!("File {} not found", creds_path.bold())));
    }

    let mut profiles: HashMap<String, Profile> = HashMap::new();


    let contents = std::fs::read_to_string(&creds_path)?;
    let contents = parse_profile_file(&contents)?;

    let mut current_profile_name: Option<String> = None;
    let mut properties: HashMap<String, Property> = HashMap::new();

    for content in &contents {
      // If line starts with "[" it means it's a profile
      if content.starts_with("[") {
        if let Some(profile_name) = &current_profile_name {

          // checking if profile already exists
          if profiles.contains_key(profile_name) {
            eprintln!("{} Profile {} is duplicated", "warn:".yellow(), profile_name.bold());
          }

          profiles.insert(
            profile_name.to_string(),
            Profile::new(profile_name.to_string(), properties.clone())?,
          );
        }
        let profile_name = read_profile_line(&content)?;
        current_profile_name = Some(profile_name);
        properties = HashMap::new();

        continue;
      }

      let (key, value) = read_property_line(&content)?;
      properties.insert(key.clone(), Property::new(key, value));
    }

    if let Some(profile_name) = &current_profile_name {
      profiles.insert(
        profile_name.to_string(),
        Profile::new(profile_name.to_string(), properties.clone())?,
      );
    }

    Ok(Self {
      profiles
    })
  }

  /// Get a reference to the profile by name
  pub fn get(&self, name: &str) -> Option<&Profile> {
    self.profiles.get(name)
  }

  /// Check if profile exists
  pub fn exists(&self, name: &str) -> anyhow::Result<bool> {
    Ok(self.profiles.contains_key(name))
  }

  /// Remove profile from ~/.aws/credentials file
  pub fn remove(&self, name: &str) -> anyhow::Result<()> {
    let dir_cert_path = creds_directory()?;
    let creds_path = format!("{}/credentials", dir_cert_path);

    if false == std::path::Path::new(&creds_path).exists() {
      return Ok(());
    }

    let profile = self.get(name);
    if let None = profile {
      return Ok(());
    }

    let contents = std::fs::read_to_string(&creds_path)?;
    let contents = parse_profile_file(&contents)?;

    let mut new_lines: Vec<&str> = Vec::new();

    let profile_properties = &profile.unwrap().properties;

    for line in contents {
      // Removing profile name
      if line.starts_with("[") {
        let profile_name = read_profile_line(line).unwrap();
        if name == profile_name {
          continue;
        }
      } else {
        // Removing properties
        let (key, value) = read_property_line(&line)?;
        // Searching if profile has this property
        if profile_properties.contains_key(&key) && profile_properties.get(&key).unwrap().value() == value {
          continue;
        }
      }

      new_lines.push(line);
    }

    std::fs::write(&creds_path, new_lines.join("\n"))?;

    Ok(())
  }
}

fn read_property_line(line: &str) -> anyhow::Result<(String, String)> {
  let mut parts = line.split("=");
  let key = match parts.next() {
    Some(key) => key.trim().to_string(),
    None => return Err(anyhow::Error::msg("Property definition must have a key"))
  };
  let value = match parts.next() {
    Some(value) => value.trim().to_string(),
    None => return Err(anyhow::Error::msg("Property definition must have a value"))
  };
  Ok((key, value))
}

fn read_profile_line(line: &str) -> anyhow::Result<String> {
  let line = line.trim();
  if !line.starts_with("[") {
    return Err(anyhow::Error::msg("Profile definition must start with '['"));
  } else if !line.ends_with("]") {
    return Err(anyhow::Error::msg("Profile definition must end with ']'"));
  }
  Ok(line.replace("[", "").replace("]", ""))
}

/// Returns a sanitised version of the profile file
fn parse_profile_file(file: &String) -> anyhow::Result<Vec<&str>> {
  let mut data = vec![];
  for line in file.lines().into_iter() {
    if is_empty_line(&line) || is_comment_line(&line) {
      continue;
    }
    data.push(line.trim());
  }
  Ok(data)
}

#[derive(Debug)]
pub struct Profile {
  name: String,
  properties: HashMap<String, Property>,
}

impl Profile {
  /// Create a new profile
  pub fn new(name: String, properties: HashMap<String, Property>) -> anyhow::Result<Self> {
    if false == validator::check_profile_name(&name) {
      return Err(anyhow::Error::msg("Profile name is not valid, it should be at least 2 characters long and only contains letters, numbers, - and _"));
    }

    Ok(Self { name, properties })
  }

  /// Stores the profile to the credentials file
  pub fn save(&self) -> anyhow::Result<()> {
    let dir_creds_path = creds_directory()?;
    let creds_path = format!("{}/credentials", dir_creds_path);

    ensure_creds_directory()?;

    if false == std::path::Path::new(&creds_path).exists() {
      return Err(anyhow::Error::msg(format!("File {} not found", creds_path.bold())));
    }

    let profiles = ProfileSet::from_file()?;

    if profiles.exists(&self.name)? {
      return Err(anyhow::Error::msg(format!("Profile {} already exists", &self.name.bold())));
    }

    let mut config = vec![format!("[{}]", &self.name)];
    for (key, prop) in &self.properties {
      config.push(format!("{} = {}", key, prop.value()));
    }

    let mut file = std::fs::OpenOptions::new().append(true).open(&creds_path)?;


    // Checking if file is empty and have a new line at the end, if there wasn't a new line
    // at the end, we add it
    let content = std::fs::read_to_string(&creds_path)?;
    if let Some(last_byte) = content.as_bytes().last() {
      if *last_byte != 10 {
        file.write_all("\n".as_bytes())?;
      }
    }

    file.write_all(config.join("\n").as_bytes())?;

    Ok(())
  }

  /// The name of this profile
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Returns a reference to the property named `name`
  pub fn get(&self, name: &str) -> Option<&str> {
    self.properties.get(name).map(|prop| prop.value())
  }

  /// Returns `aws_access_key_id` and `aws_secret_access_key` properties
  pub fn get_creds(&self) -> anyhow::Result<Credentials> {
    Ok(Credentials {
      access_key: self.get("aws_access_key_id").unwrap_or(&"".to_string()).to_string(),
      secret_key: self.get("aws_secret_access_key").unwrap_or(&"".to_string()).to_string(),
    })
  }
}

/// Key-Value property pair
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Property {
  key: String,
  value: String,
}

impl Property {
  /// Value of this property
  pub fn value(&self) -> &str {
    &self.value
  }

  /// Name of this property
  pub fn key(&self) -> &str {
    &self.key
  }

  /// Creates a new property
  pub fn new(key: String, value: String) -> Self {
    Property { key, value }
  }
}

#[cfg(test)]
mod tests_profile_management {
  use super::*;

  fn create_profile() -> anyhow::Result<Profile> {
    let profile_name = "test-profile";
    let access_key = String::from("xxxxxxxxxxxxxxxxxxxxxxxxxxx");
    let secret_key = String::from("xxxxxxxxxxxxxxxxxxxxxxxxxxx");

    let mut properties = HashMap::new();
    properties.insert(
      "aws_access_key_id".to_string(),
      Property::new("aws_access_key_id".to_string(), access_key.clone()),
    );

    properties.insert(
      "aws_secret_access_key".to_string(),
      Property::new("aws_secret_access_key".to_string(), secret_key.clone()),
    );

    Ok(Profile::new(profile_name.to_string(), properties)?)
  }


  #[test]
  #[ignore]
  fn test_read_profiles_from_file() {
    let profiles = ProfileSet::from_file().unwrap();
    for profile in &profiles.profiles {
      println!("{:?}", profile);
    }
  }

  #[test]
  #[ignore]
  fn test_profile_exist() {
    let profiles = ProfileSet::from_file().unwrap();
    let profile_name = "test-profile";

    if profiles.exists(profile_name).unwrap() {
      println!(">> {}", "Exists".green());
    } else {
      println!(">> {}", "Not Exists".red());
    }
  }

  #[test]
  #[ignore]
  fn test_create_profile() {
    let profile = create_profile();
    if let Err(e) = &profile {
      eprintln!("{e:?}");
      assert!(false, "failed to create a profile instance");
    }

    let profile = profile.unwrap();
    let result = &profile.save();
    if let Err(e) = result {
      eprintln!("error: {e:?}");
      assert!(false, "failed to save a profile");
    }
  }

  #[test]
  #[ignore]
  fn test_remove_profile() {
    let profile_name = "test-profile";

    let profiles = ProfileSet::from_file().unwrap();

    if false == profiles.exists(profile_name).unwrap() {
      panic!("test profile does not exist");
    }

    let profile = profiles.get(profile_name).unwrap();

    // print profile properties
    for property in &profile.properties {
      println!("{:?}", property);
    }

    let result = profiles.remove(profile_name);
    if let Err(e) = result {
      eprintln!("{e:?}");
      assert!(false);
    }
  }
}
