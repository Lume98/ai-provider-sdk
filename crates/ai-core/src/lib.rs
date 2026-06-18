use ai_provider::{
    AiError, EmbeddingModel, EmbeddingResult, GenerateResult, ImageModel, ImageModelCallOptions,
    ImageResult, LanguageModel, LanguageModelCallOptions, RerankingModel,
    RerankingModelCallOptions, RerankingResult, SpeechModel, SpeechModelCallOptions, SpeechResult,
    StreamResult, TranscriptionModel, TranscriptionModelCallOptions, TranscriptionResult,
};

pub async fn generate_text(
    model: &dyn LanguageModel,
    options: LanguageModelCallOptions,
) -> Result<GenerateResult, AiError> {
    model.generate(options).await
}

pub async fn stream_text(
    model: &dyn LanguageModel,
    options: LanguageModelCallOptions,
) -> Result<StreamResult, AiError> {
    model.stream(options).await
}

pub async fn embed(
    model: &dyn EmbeddingModel,
    values: Vec<String>,
) -> Result<EmbeddingResult, AiError> {
    model.embed(values).await
}

pub async fn generate_image(
    model: &dyn ImageModel,
    options: ImageModelCallOptions,
) -> Result<ImageResult, AiError> {
    model.generate_image(options).await
}

pub async fn generate_speech(
    model: &dyn SpeechModel,
    options: SpeechModelCallOptions,
) -> Result<SpeechResult, AiError> {
    model.generate_speech(options).await
}

pub async fn transcribe(
    model: &dyn TranscriptionModel,
    options: TranscriptionModelCallOptions,
) -> Result<TranscriptionResult, AiError> {
    model.transcribe(options).await
}

pub async fn rerank(
    model: &dyn RerankingModel,
    options: RerankingModelCallOptions,
) -> Result<RerankingResult, AiError> {
    model.rerank(options).await
}

#[cfg(test)]
mod tests {
    use ai_provider::{
        ContentPart, FinishReason, GenerateResult, LanguageModelStream, StreamResult, Usage,
        Warning,
    };
    use async_trait::async_trait;
    use futures::stream;

    use super::*;

    struct FakeLanguageModel;

    #[async_trait]
    impl LanguageModel for FakeLanguageModel {
        fn provider(&self) -> &str {
            "fake"
        }

        fn model_id(&self) -> &str {
            "fake-model"
        }

        async fn generate(
            &self,
            options: LanguageModelCallOptions,
        ) -> Result<GenerateResult, AiError> {
            Ok(GenerateResult {
                content: vec![ContentPart::Text {
                    text: options.prompt[0].content.clone(),
                }],
                finish_reason: FinishReason::Stop,
                usage: Usage::default(),
                response: None,
                warnings: vec![Warning::Other {
                    message: "from trait".to_string(),
                }],
            })
        }

        async fn stream(
            &self,
            _options: LanguageModelCallOptions,
        ) -> Result<StreamResult, AiError> {
            let stream: LanguageModelStream = Box::pin(stream::empty());
            Ok(StreamResult {
                stream,
                warnings: vec![],
            })
        }
    }

    #[tokio::test]
    async fn generate_text_uses_language_model_trait() {
        let result = generate_text(
            &FakeLanguageModel,
            LanguageModelCallOptions {
                prompt: vec![ai_provider::LanguageMessage::user("hello")],
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(
            result.content,
            vec![ContentPart::Text {
                text: "hello".to_string()
            }]
        );
    }
}
