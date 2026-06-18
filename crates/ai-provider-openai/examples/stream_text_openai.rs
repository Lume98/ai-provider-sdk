use ai_core::stream_text;
use ai_provider::{LanguageMessage, LanguageModelCallOptions};
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), ai_provider::AiError> {
    let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
    let model = provider.responses("gpt-4o-mini");

    let result = stream_text(
        &model,
        LanguageModelCallOptions {
            prompt: vec![LanguageMessage::user("Count to five.")],
            ..Default::default()
        },
    )
    .await?;

    let mut stream = result.stream;
    while let Some(part) = stream.next().await {
        println!("{:?}", part?);
    }

    Ok(())
}
