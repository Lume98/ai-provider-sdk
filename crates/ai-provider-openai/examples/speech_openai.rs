use ai_core::generate_speech;
use ai_provider::SpeechModelCallOptions;
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

#[tokio::main]
async fn main() -> Result<(), ai_provider::AiError> {
    let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
    let model = provider.speech("gpt-4o-mini-tts");
    let result = generate_speech(
        &model,
        SpeechModelCallOptions {
            text: "Rust provider abstractions can stay small and explicit.".to_string(),
            voice: "alloy".to_string(),
            format: Some("mp3".to_string()),
            instructions: None,
            provider_options: None,
            headers: None,
        },
    )
    .await?;
    println!(
        "generated {} bytes ({})",
        result.audio.len(),
        result.media_type
    );
    Ok(())
}
