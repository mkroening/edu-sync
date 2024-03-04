use edu_sync::{account::Account, config::Config};
use structopt::StructOpt;

use crate::util;

/// Updates the available courses in the configuration.
#[derive(Debug, StructOpt)]
pub struct Subcommand {}

impl Subcommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let mut config = Config::read().await?;

        if util::check_accounts(&config) {
            let results = config
                .accounts
                .values_mut()
                .map(|account_config| {
                    let account = Account::new(account_config.id.clone(), account_config.token);
                    let courses = tokio::spawn(async move { account.get_courses().await });
                    (account_config, courses)
                })
                .collect::<Vec<_>>();

            for (account_config, courses) in results {
                account_config.courses.update(courses.await??);
            }

            config.write().await?;
        }

        Ok(())
    }
}
