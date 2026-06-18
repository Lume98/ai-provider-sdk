use ai_core::generate_text;
use ai_provider::{LanguageMessage, LanguageModelCallOptions};
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

#[tokio::main]
async fn main() -> Result<(), ai_provider::AiError> {
    let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
    let model = provider.responses("gpt-4o-mini");

    let result = generate_text(
        &model,
        LanguageModelCallOptions {
            prompt: vec![LanguageMessage::user("Write a haiku about Rust traits.")],
            max_output_tokens: Some(80),
            ..Default::default()
        },
    )
    .await?;

    println!("{result:#?}");
    Ok(())
}
