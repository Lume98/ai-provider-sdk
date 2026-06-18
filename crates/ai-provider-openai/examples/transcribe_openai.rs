use ai_core::transcribe;
use ai_provider::TranscriptionModelCallOptions;
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

#[tokio::main]
async fn main() -> Result<(), ai_provider::AiError> {
    let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
    let model = provider.transcription("gpt-4o-mini-transcribe");
    let audio = std::fs::read("sample.wav").map_err(|error| {
        ai_provider::AiError::InvalidArgument(format!("failed to read sample.wav: {error}"))
    })?;
    let result = transcribe(
        &model,
        TranscriptionModelCallOptions {
            audio,
            file_name: "sample.wav".to_string(),
            media_type: Some("audio/wav".to_string()),
            language: Some("en".to_string()),
            prompt: None,
            provider_options: None,
            headers: None,
        },
    )
    .await?;
    println!("{result:#?}");
    Ok(())
}
