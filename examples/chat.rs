use std::env;
use std::pin::pin;

use ai_provider_sdk::{ChatCompletionCreateParams, ChatMessage, OpenAI, RequestOptions};
use futures_util::StreamExt;
use serde_json::Value;

#[tokio::main]
async fn main() -> ai_provider_sdk::Result<()> {
    let model = env::var("OPENAI_CHAT_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".to_string());
    let client = OpenAI::from_env()?;

    println!("----- standard request -----");
    let completion = client
        .chat()
        .completions()
        .create(ChatCompletionCreateParams::new(
            model.clone(),
            vec![ChatMessage::user("Say this is a test")],
        ))
        .await?;

    println!(
        "{}",
        chat_completion_text(&completion.extra).unwrap_or("<no text content>")
    );

    println!("----- streaming request -----");
    let events = client
        .chat()
        .completions()
        .create_stream(ChatCompletionCreateParams::new(
            model.clone(),
            vec![ChatMessage::user(
                "How do I output all files in a directory using Rust?",
            )],
        ))
        .await?
        .events();
    let mut events = pin!(events);

    while let Some(event) = events.next().await {
        let event = event?;
        if let Ok(data) = serde_json::from_str::<Value>(&event.data) {
            if let Some(text) = chat_delta_text(&data) {
                print!("{text}");
            }
        }
    }
    println!();

    println!("----- request options -----");
    let completion = client
        .chat()
        .completions()
        .create_with_options(
            ChatCompletionCreateParams::new(model, vec![ChatMessage::user("Say this is a test")]),
            RequestOptions::new().header("x-example-id", "chat-example"),
        )
        .await?;

    println!(
        "{}",
        chat_completion_text(&completion.extra).unwrap_or("<no text content>")
    );

    Ok(())
}

fn chat_completion_text(extra: &std::collections::HashMap<String, Value>) -> Option<&str> {
    extra
        .get("choices")?
        .as_array()?
        .first()?
        .get("message")?
        .get("content")?
        .as_str()
}

fn chat_delta_text(data: &Value) -> Option<&str> {
    data.get("choices")?
        .as_array()?
        .first()?
        .get("delta")?
        .get("content")?
        .as_str()
}
