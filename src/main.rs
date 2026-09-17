#![cfg_attr(windows, windows_subsystem = "windows")]

mod chat;
mod client;
mod config;
#[cfg(windows)]
mod host;
mod terminal;

use anyhow::Result;
use client::LlmClient;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(windows)]
    {
        let mut arguments = std::env::args().skip(1);
        if arguments.next().as_deref() != Some("--terminal") {
            return host::run();
        }
        let host_window = arguments
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        terminal::initialize(host_window)?;
    }

    let config = Config::load()?;
    let client = LlmClient::new(config.selected_profile())?;

    chat::run(client).await
}
