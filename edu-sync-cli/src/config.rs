use edu_sync::config::Config;
use structopt::StructOpt;

/// Prints the path of the configuration file.
#[derive(Debug, StructOpt)]
pub struct Subcommand {}

impl Subcommand {
    pub async fn run(self) -> anyhow::Result<()> {
        println!("{}", Config::path().display());
        Ok(())
    }
}
