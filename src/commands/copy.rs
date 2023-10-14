use clap::ArgMatches;
use colored::Colorize;
use regex::Regex;

use crate::commands::CommandOpts;
use crate::s3::ParsedS3Url;

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let bkt = crate::commands::CmdArgs::from(sub_matches).get_bucket();
  let opts = <CopyOpts as CommandOpts>::from(&sub_matches);

  if opts.verbose {
    println!("{:?}", opts);
  }

  Ok(())
}

#[derive(Clone, Debug)]
pub struct CopyOpts {
  pub verbose: bool,
  pub show_progress: bool,
  pub recursive: bool,
  pub delimiter: char,
  pub from: String,
  pub to: String,
  pub exclude: Vec<Regex>,
}

impl CommandOpts for CopyOpts {
  fn from(sub_matches: &ArgMatches) -> Self {
    let verbose = sub_matches.get_one::<bool>("verbose")
       .unwrap_or_else(|| &false)
       .clone();

    let show_progress = sub_matches.get_one::<bool>("progress")
       .unwrap_or_else(|| &true)
       .clone();

    let recursive = sub_matches.get_one::<bool>("recursive")
       .unwrap_or_else(|| &false)
       .clone();

    let args = crate::commands::CmdArgs::from(sub_matches);

    let delimiter = args.parse_delimiter();

    let exclude = args.parse_exclude();

    let from = read_required_string(&sub_matches, "FROM");
    let to = read_required_string(&sub_matches, "TO");

    // Least one of the from or to must be a valid s3 url
    if false == has_least_one_s3url(&vec![from.clone(), to.clone()]) {
      eprintln!("{} {}", "error:".red(), "At least one of the FROM or TO paths must be a valid s3 URL");
      std::process::exit(1);
    }

    Self {
      verbose,
      show_progress,
      recursive,
      delimiter,
      from,
      to,
      exclude,
    }
  }
}

fn read_required_string(sub_matches: &ArgMatches, id: &str) -> String {
  let value = sub_matches.get_one::<String>(id)
     .unwrap_or_else(|| {
       eprintln!("{} {}", "error:".red(), format!("{} is required", id));
       std::process::exit(1);
     })
     .clone();

  value
}

fn has_least_one_s3url(paths: &Vec<String>) -> bool {
  for path in paths {
    if ParsedS3Url::is_s3url(path) {
      return true;
    }
  }
  false
}

#[cfg(test)]
mod copy_tests {
  use super::*;

  #[test]
  fn test_least_find_one_s3url() {
    let from = "s3://bucket/path/to/file".to_string();
    let to = "/path/to/file".to_string();
    let paths = vec![from, to];
    assert!(has_least_one_s3url(&paths));
  }
}
