use std::io::{self, Write};

use anyhow::Result;

use crate::client::{LlmClient, Message};

#[derive(Debug, PartialEq)]
pub enum Input<'a> {
    Empty,
    Exit,
    Clear,
    Hide,
    Message(&'a str),
}

pub fn parse_input(input: &str) -> Input<'_> {
    match input.trim() {
        "" => Input::Empty,
        "/exit" | "/quit" => Input::Exit,
        "/clear" => Input::Clear,
        "/hide" => Input::Hide,
        message => Input::Message(message),
    }
}

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

        let input = match parse_input(&input) {
            Input::Empty => continue,
            Input::Exit => break,
            Input::Clear => {
                history.clear();
                println!("Conversation cleared.\n");
                continue;
            }
            Input::Hide => {
                crate::terminal::hide();
                continue;
            }
            Input::Message(input) => input,
        };

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
