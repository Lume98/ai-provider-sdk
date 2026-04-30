use std::env;

use serde_json::Value;
use vendor_sdk::{ChatCompletionCreateParams, ChatMessage, GenericCreateParams, OpenAIClient};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
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
            println!("{}", serde_json::to_string_pretty(&page)?);
        }
        "models:get" => {
            let model = required_arg(args.next(), "model")?;
            let model = client.models.retrieve(&model).await?;
            println!("{}", serde_json::to_string_pretty(&model)?);
        }
        "files:list" => {
            let page = client.files.list(&Default::default()).await?;
            println!("{}", serde_json::to_string_pretty(&page)?);
        }
        "responses:create" => {
            let model = required_arg(args.next(), "model")?;
            let input = required_arg(args.next(), "input")?;
            let response = client
                .responses
                .create(&vendor_sdk::ResponseCreateParams::new(model, input))
                .await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
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
            println!("{}", serde_json::to_string_pretty(&response)?);
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
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => print_usage(),
    }
    Ok(())
}

fn required_arg(value: Option<String>, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    value.ok_or_else(|| format!("missing required argument: {name}").into())
}

fn print_usage() {
    eprintln!(
        "usage:
  openai models:list
  openai models:get <model>
  openai files:list
  openai responses:create <model> <input>
  openai chat:create <model> <message>
  openai raw:post /images/generations '<json>'"
    );
}
