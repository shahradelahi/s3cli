use std::process;

use colored::Colorize;

use crate::s3::ParsedS3Url;

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let bkt = crate::commands::CmdArgs::from(sub_matches).get_bucket();
  let opts = DuOpts::from(&sub_matches);

  if *&opts.verbose {
    println!("{:?}", opts);
  }

  let result = bkt.du(opts.clone()).await?;

  if opts.human_readable {
    human_bytes::human_bytes(result as f64);
    process::exit(0);
  } else {
    println!("{}", result.to_string());
  }

  Ok(())
}

#[derive(Clone, Debug)]
pub struct DuOpts {
  pub verbose: bool,
  pub show_total: bool,
  pub show_progress: bool,
  pub human_readable: bool,
  pub delimiter: char,
  pub prefix: String,
}

impl DuOpts {
  pub fn from(sub_matches: &clap::ArgMatches) -> Self {
    let verbose = sub_matches.get_one::<bool>("verbose")
       .unwrap_or_else(|| &false)
       .clone();

    let show_total = sub_matches.get_one::<bool>("total")
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
        p.clone()
      }
    };

    Self {
      verbose,
      show_total,
      show_progress,
      human_readable,
      delimiter,
      prefix,
    }
  }
}
