use bytes::Bytes;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    AudioResource, BatchesResource, BetaResource, CompletionsResource, ContainersResource,
    ConversationsResource, DeletedObject, EmbeddingsResource, EvalsResource, FineTuningResource,
    ImagesResource, JsonObject, ListPage, ListParams, ModelsResource, ModerationsResource,
    RealtimeResource, RequestOptions, SkillsResource, VectorStoresResource, core::path_segment,
};
use crate::{Error, parse_api_error};

simple_json_resource!(
    CompletionsResource,
    CompletionCreateParams,
    CompletionObject,
    create,
    "/completions"
);
simple_json_resource!(
    EmbeddingsResource,
    EmbeddingCreateParams,
    EmbeddingObject,
    create,
    "/embeddings"
);
simple_json_resource!(
    ModerationsResource,
    ModerationCreateParams,
    ModerationObject,
    create,
    "/moderations"
);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub owned_by: Option<String>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

impl ModelsResource {
    pub async fn list(&self) -> Result<ListPage<Model>, Error> {
        self.core
            .json_value(
                Method::GET,
                "/models",
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn retrieve(&self, model: &str) -> Result<Model, Error> {
        self.core
            .json_value(
                Method::GET,
                &format!("/models/{}", path_segment(model)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn delete(&self, model: &str) -> Result<DeletedObject, Error> {
        self.core
            .json_value(
                Method::DELETE,
                &format!("/models/{}", path_segment(model)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CompletionCreateParams {
    pub model: String,
    pub prompt: Value,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompletionObject {
    pub id: String,
    pub object: String,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EmbeddingCreateParams {
    pub model: String,
    pub input: Value,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingObject {
    pub object: String,
    #[serde(default)]
    pub data: Vec<Value>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ModerationCreateParams {
    pub model: String,
    pub input: Value,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModerationObject {
    pub id: String,
    pub model: String,
    #[serde(default)]
    pub results: Vec<Value>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GenericCreateParams {
    #[serde(flatten)]
    pub extra: JsonObject,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericObject {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(flatten)]
    pub extra: JsonObject,
}

impl ImagesResource {
    pub async fn generate(&self, params: &GenericCreateParams) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/images/generations",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn edit(&self, params: &GenericCreateParams) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/images/edits",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }
}

impl AudioResource {
    pub async fn speech(&self, params: &GenericCreateParams) -> Result<Bytes, Error> {
        let response = self
            .core
            .json(
                Method::POST,
                "/audio/speech",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            return Err(parse_api_error(status.as_u16(), &text));
        }
        Ok(response.bytes().await?)
    }

    pub async fn transcriptions(
        &self,
        params: &GenericCreateParams,
    ) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/audio/transcriptions",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn translations(&self, params: &GenericCreateParams) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/audio/translations",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }
}

generic_crud_resource!(BatchesResource, "/batches");
generic_crud_resource!(ContainersResource, "/containers");
generic_crud_resource!(ConversationsResource, "/conversations");
generic_crud_resource!(VectorStoresResource, "/vector_stores");
generic_crud_resource!(SkillsResource, "/skills");
generic_crud_resource!(EvalsResource, "/evals");

impl FineTuningResource {
    pub async fn create_job(&self, params: &GenericCreateParams) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/fine_tuning/jobs",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn retrieve_job(&self, job_id: &str) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::GET,
                &format!("/fine_tuning/jobs/{}", path_segment(job_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn list_jobs(&self, params: &ListParams) -> Result<ListPage<GenericObject>, Error> {
        self.core
            .json_value(
                Method::GET,
                "/fine_tuning/jobs",
                Some(params),
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }

    pub async fn cancel_job(&self, job_id: &str) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                &format!("/fine_tuning/jobs/{}/cancel", path_segment(job_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RealtimeEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub data: JsonObject,
}

impl RealtimeResource {
    pub async fn create_session(
        &self,
        params: &GenericCreateParams,
    ) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/realtime/sessions",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn create_transcription_session(
        &self,
        params: &GenericCreateParams,
    ) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/realtime/transcription_sessions",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn connect(&self, _model: &str) -> Result<RealtimeConnection, Error> {
        Err(Error::Unsupported(
            "websocket transport is modeled but not enabled in this minimal build".into(),
        ))
    }
}

pub struct RealtimeConnection;

impl RealtimeConnection {
    pub async fn send(&mut self, _event: &RealtimeEvent) -> Result<(), Error> {
        Err(Error::Unsupported(
            "websocket transport is not connected".into(),
        ))
    }

    pub async fn next(&mut self) -> Result<Option<RealtimeEvent>, Error> {
        Err(Error::Unsupported(
            "websocket transport is not connected".into(),
        ))
    }
}

impl BetaResource {
    pub async fn create_assistant(
        &self,
        params: &GenericCreateParams,
    ) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::POST,
                "/assistants",
                Option::<&()>::None,
                Some(params),
                RequestOptions::default(),
            )
            .await
    }

    pub async fn retrieve_assistant(&self, assistant_id: &str) -> Result<GenericObject, Error> {
        self.core
            .json_value(
                Method::GET,
                &format!("/assistants/{}", path_segment(assistant_id)),
                Option::<&()>::None,
                Option::<&()>::None,
                RequestOptions::default(),
            )
            .await
    }
}
