use serde::{Deserialize, Serialize};
use crate::domain::translation::{TranslationConfig, TranslationConnectionStatus, TranslationRequest, TranslationResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranslationConfigDto { pub(crate) provider: String, pub(crate) api_url: String, pub(crate) app_id: String,
    pub(crate) secret_key: String, pub(crate) allow_third_party: bool }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranslationRequestDto { pub(crate) text: String, pub(crate) source_language: String, pub(crate) target_language: String }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranslationResultDto { translated_text: String, detected_language: Option<String>, character_count: usize, usage_count: u64, provider: String }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranslationConnectionStatusDto { provider: String, message: String }

impl From<TranslationConfigDto> for TranslationConfig { fn from(value: TranslationConfigDto) -> Self { Self { provider: value.provider, api_url: value.api_url,
    app_id: value.app_id, secret_key: value.secret_key, allow_third_party: value.allow_third_party } } }
impl From<TranslationRequestDto> for TranslationRequest { fn from(value: TranslationRequestDto) -> Self { Self { text: value.text,
    source_language: value.source_language, target_language: value.target_language } } }
impl From<TranslationResult> for TranslationResultDto { fn from(value: TranslationResult) -> Self { Self { translated_text: value.translated_text,
    detected_language: value.detected_language, character_count: value.character_count, usage_count: value.usage_count, provider: value.provider } } }
impl From<TranslationConnectionStatus> for TranslationConnectionStatusDto { fn from(value: TranslationConnectionStatus) -> Self {
    Self { provider: value.provider, message: value.message } } }
