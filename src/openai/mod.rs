use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type JsonObject = serde_json::Map<String, Value>;

macro_rules! resource {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name {
            pub(crate) core: Arc<core::HttpCore>,
        }

        impl $name {
            pub(crate) fn new(core: Arc<core::HttpCore>) -> Self {
                Self { core }
            }
        }
    };
}

macro_rules! simple_json_resource {
    ($resource:ident, $params:ident, $output:ident, $method:ident, $path:literal) => {
        impl $resource {
            pub async fn $method(&self, params: &$params) -> Result<$output, crate::Error> {
                self.core
                    .json_value(
                        reqwest::Method::POST,
                        $path,
                        Option::<&()>::None,
                        Some(params),
                        RequestOptions::default(),
                    )
                    .await
            }
        }
    };
}

macro_rules! generic_crud_resource {
    ($resource:ident, $base:literal) => {
        impl $resource {
            pub async fn create(
                &self,
                params: &GenericCreateParams,
            ) -> Result<GenericObject, crate::Error> {
                self.core
                    .json_value(
                        reqwest::Method::POST,
                        $base,
                        Option::<&()>::None,
                        Some(params),
                        RequestOptions::default(),
                    )
                    .await
            }

            pub async fn retrieve(&self, id: &str) -> Result<GenericObject, crate::Error> {
                self.core
                    .json_value(
                        reqwest::Method::GET,
                        &format!("{}/{}", $base, $crate::openai::core::path_segment(id)),
                        Option::<&()>::None,
                        Option::<&()>::None,
                        RequestOptions::default(),
                    )
                    .await
            }

            pub async fn list(
                &self,
                params: &ListParams,
            ) -> Result<ListPage<GenericObject>, crate::Error> {
                self.core
                    .json_value(
                        reqwest::Method::GET,
                        $base,
                        Some(params),
                        Option::<&()>::None,
                        RequestOptions::default(),
                    )
                    .await
            }

            pub async fn delete(&self, id: &str) -> Result<DeletedObject, crate::Error> {
                self.core
                    .json_value(
                        reqwest::Method::DELETE,
                        &format!("{}/{}", $base, $crate::openai::core::path_segment(id)),
                        Option::<&()>::None,
                        Option::<&()>::None,
                        RequestOptions::default(),
                    )
                    .await
            }
        }
    };
}

mod chat;
mod core;
mod files;
mod generic;
mod responses;
mod webhooks;

pub use chat::*;
pub use core::{OpenAIConfig, RequestOptions, TypedSseStream};
pub use files::*;
pub use generic::*;
pub use responses::*;

resource!(ResponsesResource);
resource!(CompletionsResource);
resource!(ModelsResource);
resource!(FilesResource);
resource!(UploadsResource);
resource!(ImagesResource);
resource!(AudioResource);
resource!(EmbeddingsResource);
resource!(ModerationsResource);
resource!(BatchesResource);
resource!(FineTuningResource);
resource!(EvalsResource);
resource!(ContainersResource);
resource!(ConversationsResource);
resource!(VectorStoresResource);
resource!(RealtimeResource);
resource!(WebhooksResource);
resource!(SkillsResource);
resource!(BetaResource);

#[derive(Clone)]
pub struct ChatResource {
    pub completions: ChatCompletionsResource,
}

impl ChatResource {
    fn new(core: Arc<core::HttpCore>) -> Self {
        Self {
            completions: ChatCompletionsResource::new(core),
        }
    }
}

resource!(ChatCompletionsResource);

#[derive(Clone)]
pub struct OpenAIClient {
    core: Arc<core::HttpCore>,
    pub responses: ResponsesResource,
    pub chat: ChatResource,
    pub completions: CompletionsResource,
    pub models: ModelsResource,
    pub files: FilesResource,
    pub uploads: UploadsResource,
    pub images: ImagesResource,
    pub audio: AudioResource,
    pub embeddings: EmbeddingsResource,
    pub moderations: ModerationsResource,
    pub batches: BatchesResource,
    pub fine_tuning: FineTuningResource,
    pub evals: EvalsResource,
    pub containers: ContainersResource,
    pub conversations: ConversationsResource,
    pub vector_stores: VectorStoresResource,
    pub realtime: RealtimeResource,
    pub webhooks: WebhooksResource,
    pub skills: SkillsResource,
    pub beta: BetaResource,
}

impl OpenAIClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::from_config(OpenAIConfig::new(api_key))
    }

    pub fn from_env() -> Result<Self, crate::Error> {
        OpenAIConfig::from_env().map(Self::from_config)
    }

    pub fn from_config(config: OpenAIConfig) -> Self {
        let core = Arc::new(core::HttpCore::new(config));
        Self {
            core: core.clone(),
            responses: ResponsesResource::new(core.clone()),
            chat: ChatResource::new(core.clone()),
            completions: CompletionsResource::new(core.clone()),
            models: ModelsResource::new(core.clone()),
            files: FilesResource::new(core.clone()),
            uploads: UploadsResource::new(core.clone()),
            images: ImagesResource::new(core.clone()),
            audio: AudioResource::new(core.clone()),
            embeddings: EmbeddingsResource::new(core.clone()),
            moderations: ModerationsResource::new(core.clone()),
            batches: BatchesResource::new(core.clone()),
            fine_tuning: FineTuningResource::new(core.clone()),
            evals: EvalsResource::new(core.clone()),
            containers: ContainersResource::new(core.clone()),
            conversations: ConversationsResource::new(core.clone()),
            vector_stores: VectorStoresResource::new(core.clone()),
            realtime: RealtimeResource::new(core.clone()),
            webhooks: WebhooksResource::new(core.clone()),
            skills: SkillsResource::new(core.clone()),
            beta: BetaResource::new(core),
        }
    }

    pub fn config(&self) -> &OpenAIConfig {
        &self.core.config
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListPage<T> {
    pub object: String,
    pub data: Vec<T>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeletedObject {
    pub id: String,
    pub object: String,
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_header_rejects_invalid_api_key() {
        let err = core::bearer_header_value("bad\r\nkey").unwrap_err();
        assert!(
            crate::Error::from(err)
                .to_string()
                .starts_with("invalid header:")
        );
    }

    #[test]
    fn config_defaults_to_official_openai_base_url() {
        let cfg = OpenAIConfig::new("sk-test");
        assert_eq!(cfg.base_url, "https://api.openai.com/v1");
    }
}
