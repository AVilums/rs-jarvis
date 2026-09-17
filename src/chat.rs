use std::io::{self, Write};

use anyhow::Result;

use crate::client::{LlmClient, Message};

pub async fn run(client: LlmClient) -> Result<()> {
    let mut history = Vec::new();
    let stdin = io::stdin();

    println!("Jarvis ready. Use /clear to reset, /hide to hide, or /exit to quit.\n");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let input = input.trim();
        match input {
            "" => continue,
            "/exit" | "/quit" => break,
            "/clear" => {
                history.clear();
                println!("Conversation cleared.\n");
                continue;
            }
            "/hide" => {
                crate::terminal::hide();
                continue;
            }
            _ => {}
        }

        history.push(Message::user(input.to_owned()));
        println!();

        match client.respond(&history).await {
            Ok(answer) => history.push(Message::assistant(answer)),
            Err(error) => {
                history.pop();
                eprintln!("Error: {error:#}");
            }
        }

        println!();
    }

    Ok(())
}
