use std::path::PathBuf;

use dialoguer::Password;
use edu_sync::{
    account::Account,
    config::{self, AccountConfig, Config},
};
use tokio::task;
use url::Url;

/// Adds a new account to the configuration.
#[derive(Debug, clap::Parser)]
pub struct Subcommand {
    /// The username of the account.
    ///
    /// If set, you will be prompted the corresponding password which will be
    /// used to retrieve the token. If unset, you must supply the token
    /// yourself.
    ///
    /// Find the token on your Moodle instance's website in your user account
    /// settings as the security key for the Moodle mobile web service. If you
    /// cannot find it, make sure that your Moodle instance supports the Moodle
    /// mobile web service, which is also required for the official Moodle app.
    /// The token is being saved your config file.
    #[structopt(short, long)]
    username: Option<String>,
    /// A language to force for resource retrieval.
    #[structopt(short, long)]
    lang: Option<String>,
    /// The URL of the Moodle instance.
    #[arg(value_hint = clap::ValueHint::Hostname)]
    url: Url,
    /// The path to download resources to.
    #[arg(value_hint = clap::ValueHint::DirPath)]
    path: PathBuf,
}

impl Subcommand {
    pub async fn run(self) -> anyhow::Result<()> {
        let config_task = tokio::spawn(Config::read());

        let token = if let Some(username) = self.username {
            let password =
                task::spawn_blocking(|| Password::new().with_prompt("Password").interact())
                    .await??;
            Account::login(&self.url, &username, &password).await?.token
        } else {
            task::spawn_blocking(|| Password::new().with_prompt("Token").interact())
                .await??
                .parse()?
        };

        let expanded_path = config::expand_path(&self.path)?;
        let account_config = AccountConfig::new(self.url, token, expanded_path, self.lang).await?;
        let mut config = config_task.await??;
        let account_name = account_config.to_string();
        config
            .accounts
            .insert(account_config.id.to_string(), account_config);
        config.write().await?;

        eprintln!("Successfully added {}", account_name);

        Ok(())
    }
}
