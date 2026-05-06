use ai_provider_sdk::OpenAI;

#[tokio::main]
async fn main() -> ai_provider_sdk::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let client = OpenAI::from_env()?;

    let models = client.models().list().await?;
    println!("total models: {}", models.data.len());
    for model in &models.data {
        println!("{} (owned by {})", model.id, model.owned_by);
    }

    Ok(())
}
