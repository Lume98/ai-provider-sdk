use ai_core::embed;
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

#[tokio::main]
async fn main() -> Result<(), ai_provider::AiError> {
    let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
    let model = provider.embedding("text-embedding-3-small");
    let result = embed(&model, vec!["AI SDK provider abstraction".to_string()]).await?;
    println!("{result:#?}");
    Ok(())
}
