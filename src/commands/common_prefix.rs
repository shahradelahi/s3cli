use colored::Colorize;

use crate::utc_datetime;
use crate::s3::bucket::Bucket;
use crate::s3::credentials::Credentials;
use crate::s3::ParsedS3Url;
use crate::utils::validator;

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let endpoint = sub_matches.get_one::<String>("endpoint-url");
  if endpoint.is_none() {
    eprintln!("{} Endpoint is required", "error:".red());
    std::process::exit(1);
  }

  let endpoint = endpoint.unwrap();
  if false == validator::is_url(endpoint) {
    eprintln!("{} Endpoint is not valid URL", "error:".red());
    std::process::exit(1);
  }

  let certs = Credentials::parse_arg_matches(&sub_matches)?;

  let bkt = Bucket::new(
    endpoint.to_owned(),
    certs.access_key,
    certs.secret_key,
  );

  // if the path wasn't defined we're going to list the list of buckets
  let path = sub_matches.get_one::<String>("PATH");
  if path.is_none() {
    let result = bkt.bkt_ls().await?;
    for bucket in result.buckets.unwrap() {
      // <creation_date> <bucket_name>
      // 2021-01-01T00:00:00.000Z bucket-name
      println!(
        "{} {}",
        utc_datetime(bucket.creation_date.unwrap()),
        bucket.name.unwrap()
      );
    }
    return Ok(());
  }

  // otherwise check if the path is a valid s3 url
  let path = path.unwrap();
  if false == ParsedS3Url::is_s3url(path) {
    eprintln!("{} Path is not a valid S3 URL", "error:".red());
    std::process::exit(1);
  }

  let delimiter = match sub_matches.get_one::<String>("delimiter") {
    Some(d) => {
      // check if a delimiter is a valid char
      if d.len() > 1 {
        eprintln!("{} Delimiter is not a valid char", "error:".red());
        std::process::exit(1);
      }
      d.chars().next().unwrap()
    }
    _ => '/'
  };

  let result = bkt.ls(
    sub_matches.get_one::<String>("PATH").expect("required").to_string(),
    delimiter,
  ).await?;

  if result.common_prefixes.is_some() {
    for prefix in result.common_prefixes.unwrap() {
      println!("{}", prefix.prefix.unwrap());
    }
  }

  Ok(())
}