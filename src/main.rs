use clap::{arg, command, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let matches = cli().get_matches();
  match matches.subcommand() {
    // List subcommand
    Some(("ls", sub_matches)) => { s3cli::commands::list::run(sub_matches).await? }
    Some(("common-prefix", sub_matches)) => { s3cli::commands::common_prefix::run(sub_matches).await? }
    // Du(Disk Usage) subcommand
    Some(("du", sub_matches)) => { s3cli::commands::du::run(sub_matches).await? }
    // Make Profile subcommand
    Some(("make-profile", sub_matches)) => { s3cli::commands::make_profile::run(sub_matches).await? }
    // Copy subcommand
    Some(("cp", sub_matches)) => { s3cli::commands::copy::run(sub_matches).await? }
    // If all subcommands are defined above, anything else is unreachable!()
    _ => unreachable!(),
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
     .author("Shahrad Elahi <https://github.com/shahradelahi>")
     // List subcommand
     .subcommand(
       Command::new("ls")
          .about("List all contents of a directory")
          .args(&connection_args)
          .arg(arg!(<PATH> "Path to list").required(false))
          .arg_required_else_help(true)
          .args([
            arg!(-r --recursive "recursively display all files including subdirectories under the given path"),
            arg!(--delimiter <DELIMITER> "delimiter to split the path"),
            arg!(-H --"human-readable" "print sizes in human readable format (e.g., 1K 234M 2G)"),
            arg!(--exclude <PATTERN> "exclude contents matching the pattern"),
            arg!(--verbose "show verbose output"),
          ])
     )
     // List the Common Prefixes subcommand
     .subcommand(
       Command::new("common-prefix")
          .about("List all common prefixes of a directory")
          .args(&connection_args)
          .arg(arg!(<PREFIX> "prefix to a path to list").required(true))
          .arg_required_else_help(true)
          .args([
            arg!(--delimiter <DELIMITER> "delimiter to split the path"),
          ])
     )
     // Make Profile subcommand
     .subcommand(
       Command::new("make-profile")
          .about("Creates a new profile")
          .args([
            arg!(--name <PROFILE> "name of the profile to create"),
            arg!(--"access-key" <ACCESS_KEY> "use access_key for connection to S3"),
            arg!(--"secret-key" <SECRET_KEY> "use security key for connection to S3"),
          ])
     )
     // Du subcommand
     .subcommand(
       Command::new("du")
          .about("Total size of a given path")
          .args(&connection_args)
          .arg(arg!(<PREFIX> "Prefix of the list").required(true))
          .arg_required_else_help(true)
          .args([
            arg!(--delimiter <DELIMITER> "delimiter to split the path"),
            arg!(--total "show total size of the path"),
            arg!(-H --"human-readable" "print sizes in human readable format (e.g., 1K 234M 2G)"),
            arg!(--progress "show progress bar"),
            arg!(--verbose "show verbose output"),
          ])
     )
     // Copy subcommand
     .subcommand(
       Command::new("cp")
          .about("Copy a file or directory")
          .args(&connection_args)
          .args([
            arg!(<FROM> "source path").required(true),
            arg!(<TO> "destination path").required(true),
            arg!(-r --recursive "recursively copy all files including subdirectories under the given path"),
            arg!(--delimiter <DELIMITER> "delimiter to split the path").required_if_eq("recursive", "true"),
            arg!(--exclude <PATTERN> "exclude contents matching the pattern"),
            arg!(--progress "show progress bar"),
            arg!(--verbose "show verbose output"),
          ])
     )
}
