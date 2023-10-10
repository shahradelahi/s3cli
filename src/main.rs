use clap::{arg, command, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let matches = cli().get_matches();
  match matches.subcommand() {
    // List subcommand
    Some(("ls", sub_matches)) => { s3cli::commands::list::run(sub_matches).await? }
    // Make Profile subcommand
    Some(("make-profile", sub_matches)) => { s3cli::commands::make_profile::run(sub_matches).await? }
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
