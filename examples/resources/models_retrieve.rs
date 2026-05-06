use std::env;

use ai_provider_sdk::OpenAI;

#[tokio::main]
async fn main() -> ai_provider_sdk::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let client = OpenAI::from_env()?;
    let model_id = env::var("OPENAI_CHAT_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".to_string());

    let model = client.models().retrieve(&model_id).await?;
    println!("id:       {}", model.id);
    println!("owned_by: {}", model.owned_by);
    println!("created:  {}", model.created);

    Ok(())
}
