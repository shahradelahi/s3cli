use colored::Colorize;


use crate::s3::ParsedS3Url;
use crate::utc_datetime;

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let args = crate::commands::CmdArgs::from(sub_matches);

  let bkt = args.get_bucket();

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

  let delimiter = args.parse_delimiter();

  let result = bkt.ls(path, &delimiter).await;

  if let Err(e) = result {
    eprintln!("{} {:?}", "error:".red(), e.to_string());
    std::process::exit(1);
  }

  for object in result.unwrap() {
    // <last_modified>  <bytes>  <object_key>
    // 2021-01-01T00:00:00.000Z  6651351  object-key
    // 2021-01-01T00:00:00.000Z   60.9KB  object-key

    let size = match sub_matches.get_one::<bool>("human-readable") {
      Some(true) => human_bytes::human_bytes(object.size as f64),
      _ => object.size.to_string()
    };

    println!(
      "{} {} {}",
      utc_datetime(object.last_modified.unwrap()),
      size,
      object.key.unwrap()
    );
  }

  Ok(())
}
