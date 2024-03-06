//! Moodle synchronization utility (CLI).

#![warn(rust_2018_idioms)]
#![warn(clippy::default_trait_access)]
#![warn(clippy::inconsistent_struct_constructor)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![deny(rustdoc::all)]

mod add;
mod config;
mod fetch;
mod sync;
mod util;

use std::env;

use clap::Parser;
use human_panic::setup_panic;
use tracing_subscriber::EnvFilter;

#[derive(Debug, clap::Parser)]
#[clap(name = "Edu Sync", author, about)]
enum Subcommand {
    Add(add::Subcommand),
    Config(config::Subcommand),
    Fetch(fetch::Subcommand),
    Sync(sync::Subcommand),
}

impl Subcommand {
    async fn run(self) -> anyhow::Result<()> {
        match self {
            Subcommand::Add(command) => command.run().await,
            Subcommand::Config(command) => command.run().await,
            Subcommand::Fetch(command) => command.run().await,
            Subcommand::Sync(command) => command.run().await,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fmt = tracing_subscriber::fmt();
    if env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
        fmt.with_env_filter(EnvFilter::try_from_default_env()?)
            .init();
    } else {
        fmt.init();
    }
    setup_panic!();

    Subcommand::parse().run().await
}
