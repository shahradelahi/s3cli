use std::io::Write;

use anyhow::Result;
use clap::{arg, command, Command};
use colored::Colorize;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<()> {
  let matches = cli().get_matches();
  match matches.subcommand() {
    // List subcommand
    Some(("ls", sub_matches)) => {
      println!(
        "Listing contents of {:?}",
        sub_matches.get_one::<String>("PATH").expect("required")
      );
    }
    // Make Profile subcommand
    Some(("make-profile", sub_matches)) => {

      let dir_cert_path = s3cli::Credentials::get_certs_directory()?;
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
          s3cli::Credentials::ensure_certs_directory()?;
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

      println!("Profile name: {:?}", name);
      println!("Access key: {:?}", access_key);
      println!("Secret key: {:?}", secret_key);

      if s3cli::Credentials::profile_exists(&name)? {
        print!("Profile {} already exists, do you want to overwrite it? [y/N] ", name.bold());
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
          println!("{}: Profile {} was not updated", "!!".red(), name.bold());
          std::process::exit(0);
        }

        s3cli::Credentials::profile_remove(name.as_str())?;
      }

      s3cli::Credentials {
        access_key: access_key.to_string(),
        secret_key: secret_key.to_string(),
      }
         .profile_save(name.as_str())?;
    }
    // If all subcommands are defined above, anything else is unreachable!()
    _ => unreachable!(),
  }
  Ok(())
}

fn read_til_regex(
  reg: Regex,
  input: &mut String,
  ask: &str,
  err: &str,
) -> Result<()> {
  print!("{}", ask);
  std::io::stdout().flush().unwrap();
  while let Ok(_) = std::io::stdin().read_line(input) {
    println!("Got: {}", &input.trim());
    if reg.is_match(&input.trim()) {
      break;
    }
    println!("{}", err);
    print!("{}", ask);
    std::io::stdout().flush().unwrap();
  }
  Ok(())
}

fn cli() -> Command {
  let connection_args = [
    arg!(-e --"endpoint-url" <ENDPOINT> "AWS Bucket endpoint").required(true),
    // profile is required if access-key and secret-key are not provided
    arg!(-p --profile <PROFILE> "AWS Profile to use").required_unless_present_any(&["access-key", "secret-key"]),
    arg!(--"access-key" <ACCESS_KEY> "use access_key for connection to S3").required_unless_present("profile"),
    arg!(--"secret-key" <SECRET_KEY> "use security key for connection to S3").required_unless_present("profile"),
  ];

  command!()
     .about("This is a command line tool for S3 with superpowers.")
     .subcommand_required(true)
     .arg_required_else_help(true)
     .allow_external_subcommands(true)
     .author("Shahrad Elahi")
     // List subcommand
     .subcommand(
       Command::new("ls")
          .about("List all contents of a directory")
          .args(&connection_args)
          .arg(arg!(<PATH> "Path to list").required(true))
          .arg_required_else_help(true)
          .args([
            arg!(-r --recursive "Recursively display all files including subdirectories under the given path"),
            arg!(-d --directory "Only show directories"),
            arg!(--exclude <PATTERN> "Exclude contents matching the pattern"),
          ])
     )
     // Make Profile subcommand
     .subcommand(
       Command::new("make-profile")
          .about("Creates a new profile")
          .args([
            arg!(--name <PROFILE> "Name of the profile to create"),
            arg!(--"access-key" <ACCESS_KEY> "use access_key for connection to S3"),
            arg!(--"secret-key" <SECRET_KEY> "use security key for connection to S3"),
          ])
     )
     // Put subcommand
     .subcommand(
       Command::new("push")
          .about("Pushes a file to the S3 bucket")
          .args(&connection_args)
          .arg(arg!(<FILE_PATH> "File to push"))
          .arg(arg!(<BUCKET> "Bucket to push to"))
          .arg_required_else_help(true)
          .args([
            arg!(-f --force "Force the push even if the remote file is newer"),
            arg!(-r --recursive "Recursively push files matching the pattern"),
            arg!(-e --exclude <PATTERN> "Exclude contents matching the pattern"),
          ])
     )
     // Get subcommand
     .subcommand(
       Command::new("pull")
          .about("Pulls a file from the S3 bucket")
          .args(&connection_args)
          .args([
            arg!(-f --force "Force the pull even if the local file is newer"),
            arg!(--"no-clobber" "Do not overwrite an existing file"),
            arg!(-r --recursive "Recursively pull all the files including subdirectories under the given path"),
            arg!(-e --exclude <PATTERN> "Exclude contents matching the pattern"),
          ])
          .arg(arg!(<file> "File to pull").required(true))
          .arg(arg!(<bucket> "Bucket to pull from").required(true))
          .arg_required_else_help(true)
     )
}
