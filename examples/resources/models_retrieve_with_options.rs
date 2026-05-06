use std::env;

use ai_provider_sdk::{OpenAI, RequestOptions};

#[tokio::main]
async fn main() -> ai_provider_sdk::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let client = OpenAI::from_env()?;
    let model_id = env::var("OPENAI_CHAT_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".to_string());

    let model = client
        .models()
        .retrieve_with_options(
            &model_id,
            RequestOptions::new().header("x-example-id", "models-example"),
        )
        .await?;

    println!("retrieved: {}", model.id);

    Ok(())
}
