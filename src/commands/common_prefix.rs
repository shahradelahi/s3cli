use colored::Colorize;

use crate::s3::ParsedS3Url;

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let args = crate::commands::CmdArgs::from(sub_matches);
  let bkt = args.get_bucket();

  let prefix = match sub_matches.get_one::<String>("PREFIX") {
    None => {
      eprintln!("{} Prefix is required", "error:".red());
      std::process::exit(1);
    }
    Some(p) => {
      if false == ParsedS3Url::is_s3url(p) {
        eprintln!("{} Prefix must be a valid s3 url", "error:".red());
        std::process::exit(1);
      }
      p.to_string()
    }
  };

  let delimiter = args.parse_delimiter();

  let result = bkt.prefixes(&prefix, &delimiter).await?;

  if result.common_prefixes.is_some() {
    for prefix in result.common_prefixes.unwrap() {
      // <CREATED
      println!("{}", prefix.prefix.unwrap());
    }
  }

  Ok(())
}
