pub async fn run(sub_matches: &clap::ArgMatches) -> anyhow::Result<()> {
  println!(
    "Listing contents of {:?}",
    sub_matches.get_one::<String>("PATH").expect("required")
  );

  Ok(())
}