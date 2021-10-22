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
                .filter_map(|account_config| {
                    let id = account_config.id.clone();
                    Account::try_from(id.clone())
                        .map(|account| (account_config, account))
                        .map_err(|err| println!("Error retrieving the token for {}: {}", id, err))
                        .ok()
                })
                .map(|(account_config, account)| {
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
