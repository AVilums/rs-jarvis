use std::io::{self, Write};

use anyhow::{Context, Result, bail};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::{Connector, Profile};

#[derive(Clone, Serialize)]
pub struct Message {
    role: &'static str,
    content: String,
}

impl Message {
    pub fn user(content: String) -> Self {
        Self {
            role: "user",
            content,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant",
            content,
        }
    }
}

pub struct LlmClient {
    http: Client,
    endpoint: String,
    api_key: Option<String>,
    model: String,
}

#[derive(Serialize)]
struct ResponseRequest<'a> {
    model: &'a str,
    input: &'a [Message],
    stream: bool,
    store: bool,
}

#[derive(Debug, Deserialize)]
pub struct StreamEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub delta: Option<String>,
    pub message: Option<String>,
}

pub fn parse_stream_line(line: &str) -> Result<Option<StreamEvent>> {
    let Some(data) = line.trim_end_matches('\r').strip_prefix("data: ") else {
        return Ok(None);
    };
    if data == "[DONE]" {
        return Ok(None);
    }

    serde_json::from_str(data)
        .map(Some)
        .context("received an invalid streaming event")
}

impl LlmClient {
    pub fn new(profile: &Profile) -> Result<Self> {
        match profile.connector {
            Connector::Openai | Connector::OpenaiCompatible => {}
        }

        Ok(Self {
            http: Client::new(),
            endpoint: format!("{}/responses", profile.base_url.trim_end_matches('/')),
            api_key: profile.api_key()?,
            model: profile.model.clone(),
        })
    }

    pub async fn respond(&self, messages: &[Message]) -> Result<String> {
        let mut request = self.http.post(&self.endpoint).json(&ResponseRequest {
            model: &self.model,
            input: messages,
            stream: true,
            store: false,
        });

        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request
            .send()
            .await
            .context("could not reach the LLM API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("LLM API returned {status}: {body}");
        }

        let mut stream = response.bytes_stream();
        let mut pending = Vec::new();
        let mut answer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("stream ended unexpectedly")?;
            pending.extend_from_slice(&chunk);

            while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
                let line = String::from_utf8(pending[..end].to_vec())
                    .context("stream contained invalid UTF-8")?;
                pending.drain(..=end);
                let Some(event) = parse_stream_line(&line)? else {
                    continue;
                };

                match event.kind.as_str() {
                    "response.output_text.delta" => {
                        if let Some(delta) = event.delta {
                            print!("{delta}");
                            io::stdout().flush()?;
                            answer.push_str(&delta);
                        }
                    }
                    "error" => bail!(
                        "LLM stream returned an error: {}",
                        event.message.unwrap_or_else(|| "unknown error".into())
                    ),
                    _ => {}
                }
            }
        }

        println!();
        Ok(answer)
    }
}
