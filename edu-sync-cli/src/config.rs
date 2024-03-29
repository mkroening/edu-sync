use edu_sync::config::Config;

/// Prints the path of the configuration file.
#[derive(Debug, clap::Parser)]
pub struct Subcommand {}

impl Subcommand {
    pub async fn run(self) -> anyhow::Result<()> {
        println!("{}", Config::path().display());
        Ok(())
    }
}
