use clap::ArgMatches;
use colored::Colorize;

use crate::s3::bucket::Bucket;
use crate::s3::credentials::Credentials;
use crate::s3::profile::ProfileSet;
use crate::utils::validator;

pub mod common_prefix;
pub mod du;
pub mod list;
pub mod make_profile;

pub struct CmdArgs {
  pub args: ArgMatches,
}

impl CmdArgs {
  pub fn from(args: &ArgMatches) -> Self {
    Self { args: args.clone() }
  }

  pub fn get_bucket(&self) -> Bucket {
    let endpoint = &self.args.get_one::<String>("endpoint-url");
    if endpoint.is_none() {
      eprintln!("{} Endpoint is required", "error:".red());
      std::process::exit(1);
    }

    let endpoint = endpoint.unwrap();
    if false == validator::is_url(endpoint) {
      eprintln!("{} Endpoint is not valid URL", "error:".red());
      std::process::exit(1);
    }

    let creds = self.get_credentials().unwrap_or_else(|e| {
      eprintln!("{} {:?}", "error:".red(), e.to_string());
      std::process::exit(1);
    });

    let bkt = Bucket::new(
      endpoint.to_owned(),
      creds.access_key,
      creds.secret_key,
    );

    bkt
  }

  pub fn get_credentials(&self) -> anyhow::Result<Credentials> {

    // There must be least a "profile" flag or "access-key" and "secret-key" flags
    let profile_name = self.args.get_one::<String>("profile");

    if let Some(profile_name) = profile_name {
      let profiles = ProfileSet::from_file()?;

      if let Some(profile) = profiles.get(profile_name) {
        return Ok(profile.get_creds()?);
      }

      return Err(anyhow::Error::msg("Profile not found"));
    }

    let access_key = self.args.get_one::<String>("access-key");
    let secret_key = self.args.get_one::<String>("secret-key");

    if let None = access_key {
      return Err(anyhow::Error::msg("Access key is required"));
    }

    if let None = secret_key {
      return Err(anyhow::Error::msg("Secret key is required"));
    }

    Ok(Credentials {
      access_key: access_key.unwrap().clone(),
      secret_key: secret_key.unwrap().clone(),
    })
  }

  pub fn parse_delimiter(&self) -> char {
    match &self.args.get_one::<String>("delimiter") {
      Some(d) => {
        // check if a delimiter is a valid char
        if d.len() > 1 {
          eprintln!("{} Delimiter is not a valid char", "error:".red());
          std::process::exit(1);
        }
        d.chars().next().unwrap()
      }
      _ => '/'
    }
  }
}
