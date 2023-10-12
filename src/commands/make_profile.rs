use std::collections::HashMap;
use std::io::Write;

use colored::Colorize;
use regex::Regex;

use crate::read_til_regex;
use crate::s3::profile::{creds_directory, ensure_creds_directory, Profile, ProfileSet, Property};

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let dir_cert_path = creds_directory()?;
  let cert_path = format!("{}/credentials", dir_cert_path);

  // Check for file "~/.aws/credentials", if it wasn't exists ask user can we create it?
  // If user said yes, create it and write config to it.
  // If user said no, exit with error.
  if false == std::path::Path::new(&cert_path).exists() {
    print!("File {} not found, do you want to create it? [Y/n] ", cert_path.bold());
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" || input.trim() == "" {
      ensure_creds_directory()?;
    } else {
      println!("{} Profile not created", "!!".yellow());
      std::process::exit(1);
    }
  }

  let name: Option<String>;
  if sub_matches.get_one::<String>("name").is_none() {
    let mut input = String::new();
    if let Err(e) = read_til_regex(
      Regex::new(r"^[a-zA-Z0-9-_]{2,}$").unwrap(),
      &mut input,
      "Enter profile name: ",
      format!("{} Profile name is not valid, it should be at least 2 characters long and only contains letters, numbers, - and _", "!!".red()).as_str(),
    ) {
      eprintln!("Something went wrong: {:?}", e);
      std::process::exit(1);
    }
    name = Some(input.trim().to_string());
  } else {
    name = sub_matches.get_one::<String>("name").cloned();
  }

  let access_key: Option<String>;
  if sub_matches.get_one::<String>("access-key").is_none() {
    let mut input = String::new();
    if let Err(e) = read_til_regex(
      Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap(),
      &mut input,
      "Enter access key: ",
      format!("{} Access key is not valid, it should be valid UUID", "!!".red()).as_str(),
    ) {
      eprintln!("Something went wrong: {:?}", e);
      std::process::exit(1);
    }
    access_key = Some(input.trim().to_string());
  } else {
    access_key = sub_matches.get_one::<String>("access-key").cloned();
  }

  let secret_key: Option<String>;
  if sub_matches.get_one::<String>("secret-key").is_none() {
    print!("Enter secret key: ");
    std::io::stdout().flush().unwrap();
    let input = rpassword::read_password().unwrap();
    secret_key = Some(input.trim().to_string());
  } else {
    secret_key = sub_matches.get_one::<String>("secret-key").cloned();
  }

  let name = name.unwrap();
  let access_key = access_key.unwrap();
  let secret_key = secret_key.unwrap();

  let profiles = ProfileSet::from_file()?;

  if profiles.exists(&name)? {
    print!("Profile {} already exists, do you want to overwrite it? [y/N] ", name.bold());
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
      println!("{}: Profile {} was not updated", "!!".red(), name.bold());
      std::process::exit(0);
    }

    profiles.remove(name.as_str())?;
  }

  let mut properties: HashMap<String, Property> = HashMap::new();

  properties.insert(
    "aws_access_key_id".to_string(),
    Property::new("aws_access_key_id".to_string(), access_key.clone()),
  );

  properties.insert(
    "aws_secret_access_key".to_string(),
    Property::new("aws_secret_access_key".to_string(), secret_key.clone()),
  );

  let new_profile = Profile::new(name.clone(), properties)?;

  if let Err(e) = new_profile.save() {
    eprintln!("{} {:?}", "error:".red(), e.to_string());
    std::process::exit(1);
  }

  Ok(())
}