use std::collections::HashMap;

use clap::ArgMatches;
use colored::Colorize;
use regex::Regex;

use crate::commands::CommandOpts;
use crate::s3::content::S3Directory;
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

  let result = result.unwrap();
  if opts.verbose {
    println!("{:?}", result);
  }

  // if it was recursive we just need to list given files
  if opts.recursive {
    print_objects(&result.objects, opts.human_readable);
    return Ok(());
  }

  print_directories(&result.directories, opts.human_readable);
  print_objects(&result.objects, opts.human_readable);

  Ok(())
}

/// Prints table of objects to stdout
fn print_objects(objs: &Vec<aws_sdk_s3::types::Object>, human_size: bool) {
  for object in objs {
    // <last_modified> <bytes> <object_key>
    // 2021-01-01T00:00:00.000Z 6651351 object-key
    // 2021-01-01T00:00:00.000Z  60.9KB object-key

    let size = match human_size {
      true => human_bytes::human_bytes(object.size as f64),
      _ => object.size.to_string()
    };

    println!(
      "{} {} {}",
      utc_datetime(object.last_modified.unwrap()),
      size,
      object.key.as_ref().unwrap()
    );
  }
}

/// Prints table of directories to stdout
fn print_directories(dirs: &HashMap<String, S3Directory>, human_size: bool) {
  for directory in dirs.values() {
    // <last_modified> DIR <bytes> <object_key>
    // 2021-01-01T00:00:00.000Z DIR  6651351  object-key
    // 2021-01-01T00:00:00.000Z DIR   60.9KB  object-key

    let size = match human_size {
      true => human_bytes::human_bytes(directory.size() as f64),
      _ => directory.size().to_string()
    };

    println!(
      "{} DIR {} {}",
      utc_datetime(directory.last_modified()),
      size,
      directory.name
    );
  }
}

#[derive(Clone, Debug)]
pub struct ListOpts {
  pub verbose: bool,
  pub recursive: bool,
  pub show_progress: bool,
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

    let show_progress = sub_matches.get_one::<bool>("progress")
       .unwrap_or_else(|| &true)
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
      show_progress,
      human_readable,
      delimiter,
      path,
      exclude,
    }
  }
}


