use std::env;

use log::{error, warn};
use serde_json::Value;
use vendor_ai_sdk::{ChatCompletionCreateParams, ChatMessage, GenericCreateParams, OpenAIClient};

#[tokio::main]
async fn main() {
    vendor_ai_sdk::init_default_logger();
    if let Err(err) = run().await {
        error!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    let client = OpenAIClient::from_env()?;
    match command.as_str() {
        "models:list" => {
            let page = client.models.list().await?;
            print_json(&page)?;
        }
        "models:get" => {
            let model = required_arg(args.next(), "model")?;
            let model = client.models.retrieve(&model).await?;
            print_json(&model)?;
        }
        "files:list" => {
            let page = client.files.list(&Default::default()).await?;
            print_json(&page)?;
        }
        "responses:create" => {
            let model = required_arg(args.next(), "model")?;
            let input = required_arg(args.next(), "input")?;
            let response = client
                .responses
                .create(&vendor_ai_sdk::ResponseCreateParams::new(model, input))
                .await?;
            print_json(&response)?;
        }
        "chat:create" => {
            let model = required_arg(args.next(), "model")?;
            let content = required_arg(args.next(), "message")?;
            let response = client
                .chat
                .completions
                .create(&ChatCompletionCreateParams::new(
                    model,
                    vec![ChatMessage::user(content)],
                ))
                .await?;
            print_json(&response)?;
        }
        "raw:post" => {
            let path = required_arg(args.next(), "path")?;
            let body = required_arg(args.next(), "json")?;
            let value: Value = serde_json::from_str(&body)?;
            let params = GenericCreateParams {
                extra: value.as_object().cloned().unwrap_or_default(),
            };
            let output = match path.as_str() {
                "/images/generations" => client.images.generate(&params).await?,
                "/batches" => client.batches.create(&params).await?,
                _ => {
                    return Err(format!("raw:post does not support {path}").into());
                }
            };
            print_json(&output)?;
        }
        _ => print_usage(),
    }
    Ok(())
}

fn required_arg(value: Option<String>, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    value.ok_or_else(|| format!("missing required argument: {name}").into())
}

fn print_usage() {
    warn!(
        "usage:
  openai models:list
  openai models:get <model>
  openai files:list
  openai responses:create <model> <input>
  openai chat:create <model> <message>
  openai raw:post /images/generations '<json>'"
    );
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
