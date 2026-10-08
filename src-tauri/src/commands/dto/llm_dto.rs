use serde::{Deserialize, Serialize};

use crate::domain::llm::{
    LlmMessage, LlmMessageRole, LlmChatRequest, LlmChatResponse, LlmConfig,
    LlmConnectionStatus,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LlmConfigDto {
    #[serde(default = "default_base_url")]
    pub(crate) base_url: String,
    #[serde(default = "default_model")]
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) api_key: String,
    #[serde(default = "default_timeout_seconds")]
    pub(crate) timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LlmChatRequestDto {
    pub(crate) messages: Vec<LlmMessageDto>,
    #[serde(default = "default_temperature")]
    pub(crate) temperature: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LlmMessageRoleDto {
    System,
    User,
    Assistant,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LlmMessageDto {
    pub(crate) role: LlmMessageRoleDto,
    pub(crate) content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LlmChatResponseDto {
    content: String,
    model: String,
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LlmConnectionStatusDto {
    provider: String,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub(crate) enum LlmStreamEventDto {
    Delta { content: String },
    Complete,
}

fn default_base_url() -> String {
    "http://127.0.0.1:8080".into()
}
fn default_model() -> String {
    "local-model".into()
}
fn default_timeout_seconds() -> u64 {
    120
}
fn default_temperature() -> f32 {
    0.2
}
impl From<LlmConfigDto> for LlmConfig {
    fn from(value: LlmConfigDto) -> Self {
        Self {
            base_url: value.base_url,
            model: value.model,
            api_key: value.api_key,
            timeout_seconds: value.timeout_seconds,
        }
    }
}

impl From<LlmMessageRoleDto> for LlmMessageRole {
    fn from(value: LlmMessageRoleDto) -> Self {
        match value {
            LlmMessageRoleDto::System => Self::System,
            LlmMessageRoleDto::User => Self::User,
            LlmMessageRoleDto::Assistant => Self::Assistant,
        }
    }
}

impl From<LlmMessageDto> for LlmMessage {
    fn from(value: LlmMessageDto) -> Self {
        Self {
            role: value.role.into(),
            content: value.content,
        }
    }
}

impl From<LlmChatRequestDto> for LlmChatRequest {
    fn from(value: LlmChatRequestDto) -> Self {
        Self {
            messages: value.messages.into_iter().map(Into::into).collect(),
            temperature: value.temperature,
        }
    }
}

impl From<LlmChatResponse> for LlmChatResponseDto {
    fn from(value: LlmChatResponse) -> Self {
        Self {
            content: value.content,
            model: value.model,
            prompt_tokens: value.prompt_tokens,
            completion_tokens: value.completion_tokens,
            total_tokens: value.total_tokens,
        }
    }
}

impl From<LlmConnectionStatus> for LlmConnectionStatusDto {
    fn from(value: LlmConnectionStatus) -> Self {
        Self {
            provider: value.provider,
            message: value.message,
        }
    }
}
