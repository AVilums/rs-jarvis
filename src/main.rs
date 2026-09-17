mod chat;
mod client;
mod config;
mod terminal;

use anyhow::Result;
use client::LlmClient;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let _terminal = terminal::initialize()?;
    let config = Config::load()?;
    let client = LlmClient::new(config.selected_profile())?;

    chat::run(client).await
}
