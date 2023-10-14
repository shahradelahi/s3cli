use std::process;

use colored::Colorize;

pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  let bkt = crate::commands::CmdArgs::from(sub_matches).get_bucket();
  let opts = DuOpts::from(&sub_matches);

  if opts.verbose {
    println!("{:?}", opts);
  }

  let result = bkt.du(opts.clone()).await;

  if let Err(e) = result {
    eprintln!("{} {:?}", "error:".red(), e.to_string());
    process::exit(1);
  }

  let size = result.unwrap() as f64;

  if opts.human_readable {
    println!("{}", human_bytes::human_bytes(size));
  } else {
    println!("{}", size.to_string());
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

    let prefix = args.parse_prefix("PREFIX", false).unwrap();

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
