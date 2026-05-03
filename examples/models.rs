use std::env;

use ai_provider_sdk::{OpenAI, RequestOptions};

#[tokio::main]
async fn main() -> ai_provider_sdk::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let client = OpenAI::from_env()?;

    println!("----- list models -----");
    let models = client.models().list().await?;
    println!("total models: {}", models.data.len());
    for model in &models.data {
        println!(
            "{} (owned by {})",
            model.id,
            model.owned_by
        );
    }

    println!("----- retrieve model -----");
    let model_id =
        env::var("OPENAI_CHAT_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".to_string());
    let model = client.models().retrieve(&model_id).await?;
    println!("id:       {}", model.id);
    println!(
        "owned_by: {}",
        model.owned_by
    );
    println!(
        "created:  {}",
        model.created
    );

    println!("----- retrieve model with request options -----");
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
