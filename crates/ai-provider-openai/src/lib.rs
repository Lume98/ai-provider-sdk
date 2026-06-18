use std::collections::BTreeMap;

use ai_provider::{
    AiError, ContentPart, EmbeddingModel, EmbeddingResult, Files, FinishReason, GenerateResult,
    Headers, ImageModel, ImageModelCallOptions, ImageResult, LanguageModel,
    LanguageModelCallOptions, LanguageModelStream, LanguageModelStreamPart, Provider, Reasoning,
    RerankingModel, ResponseFormat, ResponseMetadata, SpeechModel, SpeechModelCallOptions,
    SpeechResult, StreamResult, ToolChoice, TranscriptionModel, TranscriptionModelCallOptions,
    TranscriptionResult, Usage, Warning,
};
use ai_provider_utils::{
    load_api_key, merge_headers, parse_provider_options, post_form_to_api, post_json_to_api,
    post_json_to_api_bytes, post_json_to_api_stream, user_agent,
};
use async_trait::async_trait;
use futures::{StreamExt, stream};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct OpenAIProviderSettings {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub headers: Headers,
    pub client: reqwest::Client,
}

impl Default for OpenAIProviderSettings {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            headers: Headers::new(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    headers: Headers,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(settings: OpenAIProviderSettings) -> Result<Self, AiError> {
        let api_key = load_api_key(settings.api_key, "OPENAI_API_KEY", "OpenAI")?;
        let base_url = settings
            .base_url
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        Ok(Self {
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            headers: settings.headers,
            client: settings.client,
        })
    }

    pub fn responses(&self, model_id: impl Into<String>) -> OpenAILanguageModel {
        OpenAILanguageModel::new(self.clone(), model_id.into(), LanguageEndpoint::Responses)
    }

    pub fn chat(&self, model_id: impl Into<String>) -> OpenAILanguageModel {
        OpenAILanguageModel::new(self.clone(), model_id.into(), LanguageEndpoint::Chat)
    }

    pub fn embedding(&self, model_id: impl Into<String>) -> OpenAIEmbeddingModel {
        OpenAIEmbeddingModel {
            provider: self.clone(),
            model_id: model_id.into(),
        }
    }

    pub fn image(&self, model_id: impl Into<String>) -> OpenAIImageModel {
        OpenAIImageModel {
            provider: self.clone(),
            model_id: model_id.into(),
        }
    }

    pub fn speech(&self, model_id: impl Into<String>) -> OpenAISpeechModel {
        OpenAISpeechModel {
            provider: self.clone(),
            model_id: model_id.into(),
        }
    }

    pub fn transcription(&self, model_id: impl Into<String>) -> OpenAITranscriptionModel {
        OpenAITranscriptionModel {
            provider: self.clone(),
            model_id: model_id.into(),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn request_headers(&self, call_headers: Option<Headers>) -> Headers {
        let mut headers = BTreeMap::from([
            (
                "authorization".to_string(),
                format!("Bearer {}", self.api_key),
            ),
            ("user-agent".to_string(), user_agent()),
        ]);
        headers = merge_headers(headers, Some(self.headers.clone()));
        merge_headers(headers, call_headers)
    }
}

impl Provider for OpenAIProvider {
    fn language_model(&self, model_id: &str) -> Result<Box<dyn LanguageModel>, AiError> {
        Ok(Box::new(self.responses(model_id)))
    }

    fn embedding_model(&self, model_id: &str) -> Result<Box<dyn EmbeddingModel>, AiError> {
        Ok(Box::new(self.embedding(model_id)))
    }

    fn image_model(&self, model_id: &str) -> Result<Box<dyn ImageModel>, AiError> {
        Ok(Box::new(self.image(model_id)))
    }

    fn speech_model(&self, model_id: &str) -> Result<Box<dyn SpeechModel>, AiError> {
        Ok(Box::new(self.speech(model_id)))
    }

    fn transcription_model(&self, model_id: &str) -> Result<Box<dyn TranscriptionModel>, AiError> {
        Ok(Box::new(self.transcription(model_id)))
    }

    fn reranking_model(&self, model_id: &str) -> Result<Box<dyn RerankingModel>, AiError> {
        Err(AiError::Unsupported(format!(
            "OpenAI does not provide reranking model '{model_id}' in this adapter",
        )))
    }

    fn files(&self) -> Result<Box<dyn Files>, AiError> {
        Err(AiError::Unsupported(
            "OpenAI files are not implemented in this adapter".to_string(),
        ))
    }

    fn skills(&self) -> Result<Box<dyn ai_provider::Skills>, AiError> {
        Err(AiError::Unsupported(
            "OpenAI skills are not implemented in this adapter".to_string(),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LanguageEndpoint {
    Responses,
    Chat,
}

#[derive(Debug, Clone)]
pub struct OpenAILanguageModel {
    provider: OpenAIProvider,
    model_id: String,
    endpoint: LanguageEndpoint,
}

impl OpenAILanguageModel {
    fn new(provider: OpenAIProvider, model_id: String, endpoint: LanguageEndpoint) -> Self {
        Self {
            provider,
            model_id,
            endpoint,
        }
    }
}

#[async_trait]
impl LanguageModel for OpenAILanguageModel {
    fn provider(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn generate(&self, options: LanguageModelCallOptions) -> Result<GenerateResult, AiError> {
        let (body, warnings) =
            build_language_request_body(&self.model_id, self.endpoint, &options, false)?;
        let headers = self.provider.request_headers(options.headers);
        let endpoint = match self.endpoint {
            LanguageEndpoint::Responses => "responses",
            LanguageEndpoint::Chat => "chat/completions",
        };
        let response: Value = post_json_to_api(
            &self.provider.client,
            &self.provider.endpoint(endpoint),
            &headers,
            &body,
        )
        .await?;

        match self.endpoint {
            LanguageEndpoint::Responses => parse_responses_generate_response(response, warnings),
            LanguageEndpoint::Chat => parse_chat_generate_response(response, warnings),
        }
    }

    async fn stream(&self, options: LanguageModelCallOptions) -> Result<StreamResult, AiError> {
        let (body, warnings) =
            build_language_request_body(&self.model_id, self.endpoint, &options, true)?;
        let headers = self.provider.request_headers(options.headers);
        let endpoint = match self.endpoint {
            LanguageEndpoint::Responses => "responses",
            LanguageEndpoint::Chat => "chat/completions",
        };
        let events = post_json_to_api_stream(
            &self.provider.client,
            &self.provider.endpoint(endpoint),
            &headers,
            &body,
        )
        .await?;
        let warnings_for_stream = warnings.clone();
        let stream = events.filter_map(move |event| async move {
            match event {
                Ok(event) if event.data == "[DONE]" => None,
                Ok(event) => Some(parse_openai_stream_event(event.data)),
                Err(error) => Some(Err(error)),
            }
        });

        let start = stream::once(async move {
            Ok(LanguageModelStreamPart::StreamStart {
                warnings: warnings_for_stream,
            })
        });
        let stream: LanguageModelStream = Box::pin(start.chain(stream));

        Ok(StreamResult { stream, warnings })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAIProviderOptions {
    pub parallel_tool_calls: Option<bool>,
    pub user: Option<String>,
    pub service_tier: Option<String>,
    pub reasoning_effort: Option<String>,
}

pub fn build_language_request_body(
    model_id: &str,
    endpoint: LanguageEndpoint,
    options: &LanguageModelCallOptions,
    stream: bool,
) -> Result<(Value, Vec<Warning>), AiError> {
    let openai_options = parse_provider_options::<OpenAIProviderOptions>(
        options.provider_options.as_ref(),
        "openai",
    )?
    .unwrap_or_default();
    let mut warnings = Vec::new();

    let mut body = match endpoint {
        LanguageEndpoint::Responses => json!({
            "model": model_id,
            "input": options.prompt.iter().map(|message| {
                json!({
                    "role": message.role,
                    "content": message.content,
                })
            }).collect::<Vec<_>>(),
        }),
        LanguageEndpoint::Chat => json!({
            "model": model_id,
            "messages": options.prompt.iter().map(|message| {
                json!({
                    "role": message.role,
                    "content": message.content,
                })
            }).collect::<Vec<_>>(),
        }),
    };

    let object = body.as_object_mut().expect("request body object");
    object.insert("stream".to_string(), Value::Bool(stream));

    if let Some(value) = options.max_output_tokens {
        object.insert(
            match endpoint {
                LanguageEndpoint::Responses => "max_output_tokens".to_string(),
                LanguageEndpoint::Chat => "max_tokens".to_string(),
            },
            json!(value),
        );
    }
    if let Some(value) = options.temperature {
        object.insert("temperature".to_string(), json!(value));
    }
    if let Some(value) = options.top_p {
        object.insert("top_p".to_string(), json!(value));
    }
    if let Some(value) = &options.stop_sequences {
        object.insert("stop".to_string(), json!(value));
    }
    if let Some(response_format) = &options.response_format {
        object.insert(
            "response_format".to_string(),
            convert_response_format(response_format)?,
        );
    }
    if let Some(tools) = &options.tools {
        object.insert(
            "tools".to_string(),
            Value::Array(
                tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": tool.name,
                                "description": tool.description,
                                "parameters": tool.parameters,
                            }
                        })
                    })
                    .collect(),
            ),
        );
    }
    if let Some(tool_choice) = &options.tool_choice {
        object.insert("tool_choice".to_string(), convert_tool_choice(tool_choice));
    }
    if let Some(reasoning) = &options.reasoning {
        match reasoning {
            Reasoning::ProviderDefault => {}
            Reasoning::None => {
                warnings.push(Warning::unsupported("reasoning=none"));
            }
            Reasoning::Minimal
            | Reasoning::Low
            | Reasoning::Medium
            | Reasoning::High
            | Reasoning::XHigh => {
                let effort = openai_options
                    .reasoning_effort
                    .clone()
                    .unwrap_or_else(|| reasoning_to_openai_effort(reasoning).to_string());
                object.insert("reasoning".to_string(), json!({ "effort": effort }));
            }
        }
    }
    if let Some(value) = openai_options.parallel_tool_calls {
        object.insert("parallel_tool_calls".to_string(), json!(value));
    }
    if let Some(value) = openai_options.user {
        object.insert("user".to_string(), json!(value));
    }
    if let Some(value) = openai_options.service_tier {
        object.insert("service_tier".to_string(), json!(value));
    }

    Ok((body, warnings))
}

fn convert_response_format(response_format: &ResponseFormat) -> Result<Value, AiError> {
    Ok(match response_format {
        ResponseFormat::Text => json!({ "type": "text" }),
        ResponseFormat::Json {
            schema: None,
            name: _,
            description: _,
        } => json!({ "type": "json_object" }),
        ResponseFormat::Json {
            schema: Some(schema),
            name,
            description,
        } => {
            ai_provider_utils::validate_json_schema(schema)?;
            json!({
                "type": "json_schema",
                "json_schema": {
                    "name": name.clone().unwrap_or_else(|| "response".to_string()),
                    "description": description,
                    "schema": schema,
                }
            })
        }
    })
}

fn convert_tool_choice(tool_choice: &ToolChoice) -> Value {
    match tool_choice {
        ToolChoice::Auto => json!("auto"),
        ToolChoice::None => json!("none"),
        ToolChoice::Required => json!("required"),
        ToolChoice::Tool { name } => json!({
            "type": "function",
            "function": { "name": name },
        }),
    }
}

fn reasoning_to_openai_effort(reasoning: &Reasoning) -> &'static str {
    match reasoning {
        Reasoning::Minimal | Reasoning::Low => "low",
        Reasoning::Medium => "medium",
        Reasoning::High | Reasoning::XHigh => "high",
        Reasoning::ProviderDefault | Reasoning::None => "medium",
    }
}

fn parse_responses_generate_response(
    response: Value,
    warnings: Vec<Warning>,
) -> Result<GenerateResult, AiError> {
    let text = response
        .get("output_text")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            response
                .get("output")
                .and_then(Value::as_array)
                .and_then(|items| {
                    items.iter().find_map(|item| {
                        item.get("content")?.as_array()?.iter().find_map(|content| {
                            content
                                .get("text")
                                .and_then(Value::as_str)
                                .map(ToString::to_string)
                        })
                    })
                })
        })
        .unwrap_or_default();

    Ok(GenerateResult {
        content: vec![ContentPart::Text { text }],
        finish_reason: FinishReason::Stop,
        usage: parse_usage(response.get("usage")),
        response: Some(ResponseMetadata {
            id: response
                .get("id")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            model_id: response
                .get("model")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            timestamp: None,
            headers: None,
        }),
        warnings,
    })
}

fn parse_chat_generate_response(
    response: Value,
    warnings: Vec<Warning>,
) -> Result<GenerateResult, AiError> {
    let choice = response
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| AiError::ResponseParsing("missing OpenAI chat choice".to_string()))?;
    let text = choice
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    Ok(GenerateResult {
        content: vec![ContentPart::Text { text }],
        finish_reason: parse_finish_reason(choice.get("finish_reason").and_then(Value::as_str)),
        usage: parse_usage(response.get("usage")),
        response: Some(ResponseMetadata {
            id: response
                .get("id")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            model_id: response
                .get("model")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            timestamp: None,
            headers: None,
        }),
        warnings,
    })
}

pub fn parse_openai_stream_event(data: String) -> Result<LanguageModelStreamPart, AiError> {
    let value: Value = serde_json::from_str(&data).map_err(|error| {
        AiError::ResponseParsing(format!("invalid OpenAI stream JSON: {error}"))
    })?;

    if value.get("type").and_then(Value::as_str) == Some("response.output_text.delta") {
        return Ok(LanguageModelStreamPart::TextDelta {
            id: value
                .get("item_id")
                .and_then(Value::as_str)
                .unwrap_or("0")
                .to_string(),
            delta: value
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        });
    }

    if let Some(delta) = value.get("delta").and_then(Value::as_str) {
        return Ok(LanguageModelStreamPart::TextDelta {
            id: "0".to_string(),
            delta: delta.to_string(),
        });
    }

    if let Some(choice) = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
    {
        if let Some(content) = choice
            .get("delta")
            .and_then(|delta| delta.get("content"))
            .and_then(Value::as_str)
        {
            return Ok(LanguageModelStreamPart::TextDelta {
                id: "0".to_string(),
                delta: content.to_string(),
            });
        }

        if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
            return Ok(LanguageModelStreamPart::Finish {
                usage: Usage::default(),
                finish_reason: parse_finish_reason(Some(reason)),
            });
        }
    }

    Ok(LanguageModelStreamPart::Raw { value })
}

#[derive(Debug, Clone)]
pub struct OpenAIEmbeddingModel {
    provider: OpenAIProvider,
    model_id: String,
}

#[async_trait]
impl EmbeddingModel for OpenAIEmbeddingModel {
    fn provider(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn embed(&self, values: Vec<String>) -> Result<EmbeddingResult, AiError> {
        let body = build_embedding_request_body(&self.model_id, &values);
        let headers = self.provider.request_headers(None);
        let response: Value = post_json_to_api(
            &self.provider.client,
            &self.provider.endpoint("embeddings"),
            &headers,
            &body,
        )
        .await?;

        let embeddings = response
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| AiError::ResponseParsing("missing embeddings data".to_string()))?
            .iter()
            .map(|item| {
                item.get("embedding")
                    .and_then(Value::as_array)
                    .ok_or_else(|| AiError::ResponseParsing("missing embedding".to_string()))?
                    .iter()
                    .map(|value| {
                        value.as_f64().map(|value| value as f32).ok_or_else(|| {
                            AiError::ResponseParsing("embedding value must be a number".to_string())
                        })
                    })
                    .collect()
            })
            .collect::<Result<Vec<Vec<f32>>, AiError>>()?;

        Ok(EmbeddingResult {
            embeddings,
            usage: parse_usage(response.get("usage")),
            response: response_metadata(&response),
            warnings: vec![],
        })
    }
}

pub fn build_embedding_request_body(model_id: &str, values: &[String]) -> Value {
    json!({
        "model": model_id,
        "input": values,
    })
}

#[derive(Debug, Clone)]
pub struct OpenAIImageModel {
    provider: OpenAIProvider,
    model_id: String,
}

#[async_trait]
impl ImageModel for OpenAIImageModel {
    fn provider(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn generate_image(&self, options: ImageModelCallOptions) -> Result<ImageResult, AiError> {
        let body = build_image_request_body(&self.model_id, &options);
        let headers = self.provider.request_headers(options.headers);
        let response: Value = post_json_to_api(
            &self.provider.client,
            &self.provider.endpoint("images/generations"),
            &headers,
            &body,
        )
        .await?;
        let images = response
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| AiError::ResponseParsing("missing image data".to_string()))?
            .iter()
            .filter_map(|item| {
                item.get("b64_json")
                    .or_else(|| item.get("url"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .collect();

        Ok(ImageResult {
            images,
            response: response_metadata(&response),
            warnings: vec![],
        })
    }
}

pub fn build_image_request_body(model_id: &str, options: &ImageModelCallOptions) -> Value {
    let mut body = json!({
        "model": model_id,
        "prompt": options.prompt,
    });
    let object = body.as_object_mut().expect("image body object");
    if let Some(n) = options.n {
        object.insert("n".to_string(), json!(n));
    }
    if let Some(size) = &options.size {
        object.insert("size".to_string(), json!(size));
    }
    if let Some(response_format) = &options.response_format {
        object.insert("response_format".to_string(), json!(response_format));
    }
    body
}

#[derive(Debug, Clone)]
pub struct OpenAISpeechModel {
    provider: OpenAIProvider,
    model_id: String,
}

#[async_trait]
impl SpeechModel for OpenAISpeechModel {
    fn provider(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn generate_speech(
        &self,
        options: SpeechModelCallOptions,
    ) -> Result<SpeechResult, AiError> {
        let body = build_speech_request_body(&self.model_id, &options);
        let media_type = format_to_media_type(options.format.as_deref());
        let headers = self.provider.request_headers(options.headers);
        let (audio, headers) = post_json_to_api_bytes(
            &self.provider.client,
            &self.provider.endpoint("audio/speech"),
            &headers,
            &body,
        )
        .await?;

        Ok(SpeechResult {
            audio,
            media_type,
            response: Some(ResponseMetadata {
                id: None,
                model_id: Some(self.model_id.clone()),
                timestamp: None,
                headers: Some(headers),
            }),
            warnings: vec![],
        })
    }
}

pub fn build_speech_request_body(model_id: &str, options: &SpeechModelCallOptions) -> Value {
    let mut body = json!({
        "model": model_id,
        "input": options.text,
        "voice": options.voice,
    });
    let object = body.as_object_mut().expect("speech body object");
    if let Some(format) = &options.format {
        object.insert("response_format".to_string(), json!(format));
    }
    if let Some(instructions) = &options.instructions {
        object.insert("instructions".to_string(), json!(instructions));
    }
    body
}

#[derive(Debug, Clone)]
pub struct OpenAITranscriptionModel {
    provider: OpenAIProvider,
    model_id: String,
}

#[async_trait]
impl TranscriptionModel for OpenAITranscriptionModel {
    fn provider(&self) -> &str {
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn transcribe(
        &self,
        options: TranscriptionModelCallOptions,
    ) -> Result<TranscriptionResult, AiError> {
        let form = build_transcription_form(&self.model_id, &options)?;
        let headers = self.provider.request_headers(options.headers);
        let response: Value = post_form_to_api(
            &self.provider.client,
            &self.provider.endpoint("audio/transcriptions"),
            &headers,
            form,
        )
        .await?;

        let text = response
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| AiError::ResponseParsing("missing transcription text".to_string()))?
            .to_string();

        Ok(TranscriptionResult {
            text,
            response: response_metadata(&response),
            warnings: vec![],
        })
    }
}

pub fn build_transcription_form(
    model_id: &str,
    options: &TranscriptionModelCallOptions,
) -> Result<Form, AiError> {
    let mut file_part = Part::bytes(options.audio.clone()).file_name(options.file_name.clone());
    if let Some(media_type) = &options.media_type {
        file_part = file_part.mime_str(media_type).map_err(|error| {
            AiError::InvalidArgument(format!("invalid transcription media type: {error}"))
        })?;
    }
    let mut form = Form::new()
        .text("model", model_id.to_string())
        .part("file", file_part);
    if let Some(language) = &options.language {
        form = form.text("language", language.clone());
    }
    if let Some(prompt) = &options.prompt {
        form = form.text("prompt", prompt.clone());
    }
    Ok(form)
}

fn parse_usage(value: Option<&Value>) -> Usage {
    let Some(value) = value else {
        return Usage::default();
    };
    Usage {
        input_tokens: value
            .get("input_tokens")
            .or_else(|| value.get("prompt_tokens"))
            .and_then(Value::as_u64),
        output_tokens: value
            .get("output_tokens")
            .or_else(|| value.get("completion_tokens"))
            .and_then(Value::as_u64),
        total_tokens: value.get("total_tokens").and_then(Value::as_u64),
    }
}

fn parse_finish_reason(value: Option<&str>) -> FinishReason {
    match value {
        Some("stop") => FinishReason::Stop,
        Some("length") => FinishReason::Length,
        Some("tool_calls") => FinishReason::ToolCalls,
        Some("content_filter") => FinishReason::ContentFilter,
        Some(_) => FinishReason::Unknown,
        None => FinishReason::Unknown,
    }
}

fn response_metadata(response: &Value) -> Option<ResponseMetadata> {
    Some(ResponseMetadata {
        id: response
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        model_id: response
            .get("model")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        timestamp: None,
        headers: None,
    })
}

fn format_to_media_type(format: Option<&str>) -> String {
    match format {
        Some("opus") => "audio/opus".to_string(),
        Some("aac") => "audio/aac".to_string(),
        Some("flac") => "audio/flac".to_string(),
        Some("wav") => "audio/wav".to_string(),
        Some("pcm") => "audio/pcm".to_string(),
        Some("mp3") | None => "audio/mpeg".to_string(),
        Some(other) => format!("audio/{other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_provider::{FunctionTool, LanguageMessage};
    use serde_json::json;

    #[test]
    fn parses_valid_openai_provider_options() {
        let options = BTreeMap::from([(
            "openai".to_string(),
            json!({
                "parallelToolCalls": true,
                "user": "user-1",
                "serviceTier": "auto",
            }),
        )]);

        let parsed =
            parse_provider_options::<OpenAIProviderOptions>(Some(&options), "openai").unwrap();

        assert_eq!(
            parsed,
            Some(OpenAIProviderOptions {
                parallel_tool_calls: Some(true),
                user: Some("user-1".to_string()),
                service_tier: Some("auto".to_string()),
                reasoning_effort: None,
            })
        );
    }

    #[test]
    fn rejects_invalid_openai_provider_options() {
        let options = BTreeMap::from([("openai".to_string(), json!("bad"))]);

        let error =
            parse_provider_options::<OpenAIProviderOptions>(Some(&options), "openai").unwrap_err();

        assert!(matches!(error, AiError::InvalidArgument(_)));
    }

    #[test]
    fn unsupported_reasoning_none_produces_warning() {
        let (_, warnings) = build_language_request_body(
            "gpt-test",
            LanguageEndpoint::Responses,
            &LanguageModelCallOptions {
                prompt: vec![LanguageMessage::user("hello")],
                reasoning: Some(Reasoning::None),
                ..Default::default()
            },
            false,
        )
        .unwrap();

        assert_eq!(warnings, vec![Warning::unsupported("reasoning=none")]);
    }

    #[test]
    fn builds_language_request_body() {
        let (body, warnings) = build_language_request_body(
            "gpt-test",
            LanguageEndpoint::Responses,
            &LanguageModelCallOptions {
                prompt: vec![LanguageMessage::system("a"), LanguageMessage::user("b")],
                max_output_tokens: Some(100),
                temperature: Some(0.4),
                top_p: Some(0.9),
                stop_sequences: Some(vec!["stop".to_string()]),
                response_format: Some(ai_provider::ResponseFormat::Json {
                    schema: Some(json!({"type": "object"})),
                    name: Some("answer".to_string()),
                    description: None,
                }),
                tools: Some(vec![FunctionTool {
                    name: "lookup".to_string(),
                    description: Some("lookup things".to_string()),
                    parameters: json!({"type": "object"}),
                }]),
                tool_choice: Some(ToolChoice::Tool {
                    name: "lookup".to_string(),
                }),
                reasoning: Some(Reasoning::High),
                ..Default::default()
            },
            false,
        )
        .unwrap();

        assert!(warnings.is_empty());
        assert_eq!(body["model"], "gpt-test");
        assert_eq!(body["max_output_tokens"], 100);
        assert_eq!(body["response_format"]["type"], "json_schema");
        assert_eq!(body["tools"][0]["function"]["name"], "lookup");
        assert_eq!(body["tool_choice"]["function"]["name"], "lookup");
        assert_eq!(body["reasoning"]["effort"], "high");
    }

    #[test]
    fn builds_embedding_image_speech_requests() {
        assert_eq!(
            build_embedding_request_body("text-embedding", &["a".to_string()]),
            json!({ "model": "text-embedding", "input": ["a"] })
        );

        assert_eq!(
            build_image_request_body(
                "gpt-image",
                &ImageModelCallOptions {
                    prompt: "draw".to_string(),
                    n: Some(2),
                    size: Some("1024x1024".to_string()),
                    response_format: Some("b64_json".to_string()),
                    provider_options: None,
                    headers: None,
                }
            ),
            json!({
                "model": "gpt-image",
                "prompt": "draw",
                "n": 2,
                "size": "1024x1024",
                "response_format": "b64_json",
            })
        );

        assert_eq!(
            build_speech_request_body(
                "gpt-4o-mini-tts",
                &SpeechModelCallOptions {
                    text: "hello".to_string(),
                    voice: "alloy".to_string(),
                    format: Some("mp3".to_string()),
                    instructions: Some("calm".to_string()),
                    provider_options: None,
                    headers: None,
                }
            ),
            json!({
                "model": "gpt-4o-mini-tts",
                "input": "hello",
                "voice": "alloy",
                "response_format": "mp3",
                "instructions": "calm",
            })
        );
    }

    #[test]
    fn builds_transcription_form_and_rejects_invalid_media_type() {
        let form = build_transcription_form(
            "gpt-transcribe",
            &TranscriptionModelCallOptions {
                audio: b"audio".to_vec(),
                file_name: "sample.wav".to_string(),
                media_type: Some("audio/wav".to_string()),
                language: Some("en".to_string()),
                prompt: Some("technical terms".to_string()),
                provider_options: None,
                headers: None,
            },
        )
        .unwrap();

        assert!(!form.boundary().is_empty());

        let error = build_transcription_form(
            "gpt-transcribe",
            &TranscriptionModelCallOptions {
                audio: b"audio".to_vec(),
                file_name: "sample.wav".to_string(),
                media_type: Some("not a media type".to_string()),
                language: None,
                prompt: None,
                provider_options: None,
                headers: None,
            },
        )
        .unwrap_err();

        assert!(matches!(error, AiError::InvalidArgument(_)));
    }

    #[test]
    fn converts_sse_chunk_into_stream_part() {
        let part = parse_openai_stream_event(
            json!({
                "type": "response.output_text.delta",
                "item_id": "msg_1",
                "delta": "hello"
            })
            .to_string(),
        )
        .unwrap();

        assert_eq!(
            part,
            LanguageModelStreamPart::TextDelta {
                id: "msg_1".to_string(),
                delta: "hello".to_string(),
            }
        );
    }
}
