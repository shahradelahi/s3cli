use clap::ArgMatches;
use colored::Colorize;
use regex::Regex;

use crate::commands::CommandOpts;
use crate::utc_datetime;

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let bkt = crate::commands::CmdArgs::from(sub_matches).get_bucket();
  let opts = <ListOpts as CommandOpts>::from(&sub_matches);

  if opts.verbose {
    println!("{:?}", opts);
  }

  // if the path wasn't defined we're going to list the list of buckets
  if opts.path.is_none() {
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

  let result = bkt.ls(opts.clone()).await;

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


#[derive(Clone, Debug)]
pub struct ListOpts {
  pub verbose: bool,
  pub recursive: bool,
  pub human_readable: bool,
  pub delimiter: char,
  pub path: Option<String>,
  pub exclude: Vec<Regex>,
}

impl CommandOpts for ListOpts {
  fn from(sub_matches: &ArgMatches) -> Self {
    let verbose = sub_matches.get_one::<bool>("verbose")
       .unwrap_or_else(|| &false)
       .clone();

    let recursive = sub_matches.get_one::<bool>("recursive")
       .unwrap_or_else(|| &false)
       .clone();

    let human_readable = sub_matches.get_one::<bool>("human-readable")
       .unwrap_or_else(|| &true)
       .clone();

    let args = crate::commands::CmdArgs::from(sub_matches);

    let delimiter = args.parse_delimiter();

    let path = args.parse_prefix("PATH", true);

    let exclude = args.parse_exclude();

    Self {
      verbose,
      recursive,
      human_readable,
      delimiter,
      path,
      exclude,
    }
  }
}
