use ai_core::generate_image;
use ai_provider::ImageModelCallOptions;
use ai_provider_openai::{OpenAIProvider, OpenAIProviderSettings};

#[tokio::main]
async fn main() -> Result<(), ai_provider::AiError> {
    let provider = OpenAIProvider::new(OpenAIProviderSettings::default())?;
    let model = provider.image("gpt-image-1");
    let result = generate_image(
        &model,
        ImageModelCallOptions {
            prompt: "A clean architecture diagram for a Rust AI provider SDK".to_string(),
            n: Some(1),
            size: Some("1024x1024".to_string()),
            response_format: Some("b64_json".to_string()),
            provider_options: None,
            headers: None,
        },
    )
    .await?;
    println!("{result:#?}");
    Ok(())
}
