// mod config

mod config;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let config = Config::parse();
    //
    // let client = LlmClient::new(
    //     config.api_key,
    //     config.base_url,
    //     config.model,
    // );
    //
    // chat::run(client, config.system).await
}