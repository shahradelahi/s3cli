use std::io::Write;

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
