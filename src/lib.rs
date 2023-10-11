use std::io::Write;
use chrono::TimeZone;

use regex::Regex;

pub mod commands;
pub mod error;
pub mod s3;
pub mod utils;

pub fn read_til_regex(
  reg: Regex,
  input: &mut String,
  ask: &str,
  err: &str,
) -> anyhow::Result<()> {
  print!("{}", ask);
  std::io::stdout().flush().unwrap();
  while let Ok(_) = std::io::stdin().read_line(input) {
    if reg.is_match(&input.trim()) {
      break;
    }
    eprintln!("{}", err);
    std::io::stdout().flush().unwrap();
  }
  Ok(())
}

// converts DateTime { seconds: 1678997191, subsecond_nanos: 341000000 } to timestampz (2023-02-13 12:59:51.341 UTC)
pub fn utc_datetime(datetime: aws_sdk_s3::primitives::DateTime) -> String {
  let datetime = chrono::NaiveDateTime::from_timestamp_opt(
    datetime.secs(),
    datetime.subsec_nanos(),
  ).unwrap();
  chrono::Utc.from_utc_datetime(&datetime).to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
