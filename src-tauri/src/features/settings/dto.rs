use serde::{Deserialize, Serialize};

use crate::features::settings::AppSettings;
use crate::features::settings::DocumentImportMode;
use crate::features::settings::StartupBehavior;
use crate::features::settings::ThemePreference;
use crate::features::settings::WorkspacePanelLayout;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DocumentImportModeDto {
    #[default]
    Copy,
    Link,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ThemePreferenceDto {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) enum StartupBehaviorDto {
    #[default]
    RestoreWorkspace,
    NewMarkdown,
    StartPage,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspacePanelLayoutDto {
    pub(crate) left_width: u32,
    pub(crate) right_width: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettingsDto {
    pub(crate) annotation_storage_dir: Option<String>,
    pub(crate) export_dir: Option<String>,
    pub(crate) pdf_storage_dir: Option<String>,
    pub(crate) markdown_storage_dir: Option<String>,
    #[serde(default)]
    pub(crate) pdf_import_mode: DocumentImportModeDto,
    #[serde(default)]
    pub(crate) markdown_import_mode: DocumentImportModeDto,
    #[serde(default)]
    pub(crate) theme: ThemePreferenceDto,
    #[serde(default = "default_true")]
    pub(crate) animations_enabled: bool,
    #[serde(default = "default_true")]
    pub(crate) confirmation_reminders_enabled: bool,
    #[serde(default)]
    pub(crate) startup_behavior: StartupBehaviorDto,
    #[serde(default = "default_pdf_workspace_layout")]
    pub(crate) pdf_workspace_layout: WorkspacePanelLayoutDto,
    #[serde(default = "default_markdown_workspace_layout")]
    pub(crate) markdown_workspace_layout: WorkspacePanelLayoutDto,
    #[serde(default = "default_translation_provider")]
    pub(crate) translation_provider: String,
    #[serde(default = "default_translation_api_url")]
    pub(crate) translation_api_url: String,
    #[serde(default)]
    pub(crate) translation_api_key: String,
    #[serde(default)]
    pub(crate) translation_secret_id: String,
    #[serde(default)]
    pub(crate) translation_secret_key: String,
    #[serde(default = "default_translation_target_language")]
    pub(crate) translation_target_language: String,
    #[serde(default)]
    pub(crate) allow_third_party_translation: bool,
    #[serde(default = "default_llm_base_url", alias = "localLlmBaseUrl")]
    pub(crate) llm_base_url: String,
    #[serde(default = "default_llm_model", alias = "localLlmModel")]
    pub(crate) llm_model: String,
    #[serde(default, alias = "localLlmApiKey")]
    pub(crate) llm_api_key: String,
    #[serde(
        default = "default_llm_timeout_seconds",
        alias = "localLlmTimeoutSeconds"
    )]
    pub(crate) llm_timeout_seconds: u64,
    #[serde(default = "default_llm_temperature", alias = "localLlmTemperature")]
    pub(crate) llm_temperature: f32,
}

impl Default for AppSettingsDto {
    fn default() -> Self {
        AppSettings::default().into()
    }
}

impl From<AppSettingsDto> for AppSettings {
    fn from(value: AppSettingsDto) -> Self {
        Self {
            annotation_storage_dir: value.annotation_storage_dir,
            export_dir: value.export_dir,
            pdf_storage_dir: value.pdf_storage_dir,
            markdown_storage_dir: value.markdown_storage_dir,
            pdf_import_mode: value.pdf_import_mode.into(),
            markdown_import_mode: value.markdown_import_mode.into(),
            theme: value.theme.into(),
            animations_enabled: value.animations_enabled,
            confirmation_reminders_enabled: value.confirmation_reminders_enabled,
            startup_behavior: value.startup_behavior.into(),
            pdf_workspace_layout: value.pdf_workspace_layout.into(),
            markdown_workspace_layout: value.markdown_workspace_layout.into(),
            translation_provider: value.translation_provider,
            translation_api_url: value.translation_api_url,
            translation_api_key: value.translation_api_key,
            translation_secret_id: value.translation_secret_id,
            translation_secret_key: value.translation_secret_key,
            translation_target_language: value.translation_target_language,
            allow_third_party_translation: value.allow_third_party_translation,
            llm_base_url: value.llm_base_url,
            llm_model: value.llm_model,
            llm_api_key: value.llm_api_key,
            llm_timeout_seconds: value.llm_timeout_seconds,
            llm_temperature: value.llm_temperature,
        }
    }
}

impl From<AppSettings> for AppSettingsDto {
    fn from(value: AppSettings) -> Self {
        Self {
            annotation_storage_dir: value.annotation_storage_dir,
            export_dir: value.export_dir,
            pdf_storage_dir: value.pdf_storage_dir,
            markdown_storage_dir: value.markdown_storage_dir,
            pdf_import_mode: value.pdf_import_mode.into(),
            markdown_import_mode: value.markdown_import_mode.into(),
            theme: value.theme.into(),
            animations_enabled: value.animations_enabled,
            confirmation_reminders_enabled: value.confirmation_reminders_enabled,
            startup_behavior: value.startup_behavior.into(),
            pdf_workspace_layout: value.pdf_workspace_layout.into(),
            markdown_workspace_layout: value.markdown_workspace_layout.into(),
            translation_provider: value.translation_provider,
            translation_api_url: value.translation_api_url,
            translation_api_key: value.translation_api_key,
            translation_secret_id: value.translation_secret_id,
            translation_secret_key: value.translation_secret_key,
            translation_target_language: value.translation_target_language,
            allow_third_party_translation: value.allow_third_party_translation,
            llm_base_url: value.llm_base_url,
            llm_model: value.llm_model,
            llm_api_key: value.llm_api_key,
            llm_timeout_seconds: value.llm_timeout_seconds,
            llm_temperature: value.llm_temperature,
        }
    }
}

impl From<DocumentImportModeDto> for DocumentImportMode {
    fn from(value: DocumentImportModeDto) -> Self {
        match value {
            DocumentImportModeDto::Copy => Self::Copy,
            DocumentImportModeDto::Link => Self::Link,
        }
    }
}

impl From<DocumentImportMode> for DocumentImportModeDto {
    fn from(value: DocumentImportMode) -> Self {
        match value {
            DocumentImportMode::Copy => Self::Copy,
            DocumentImportMode::Link => Self::Link,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_translation_provider() -> String {
    "baidu".into()
}
fn default_translation_api_url() -> String {
    "https://fanyi-api.baidu.com/api/trans/vip/translate".into()
}
fn default_translation_target_language() -> String {
    "zh".into()
}
fn default_llm_base_url() -> String {
    "http://127.0.0.1:8080".into()
}
fn default_llm_model() -> String {
    "local-model".into()
}
fn default_llm_timeout_seconds() -> u64 {
    120
}
fn default_llm_temperature() -> f32 {
    0.2
}

fn default_pdf_workspace_layout() -> WorkspacePanelLayoutDto {
    WorkspacePanelLayout::pdf_default().into()
}

fn default_markdown_workspace_layout() -> WorkspacePanelLayoutDto {
    WorkspacePanelLayout::markdown_default().into()
}

impl From<WorkspacePanelLayoutDto> for WorkspacePanelLayout {
    fn from(value: WorkspacePanelLayoutDto) -> Self {
        Self {
            left_width: value.left_width,
            right_width: value.right_width,
        }
    }
}

impl From<WorkspacePanelLayout> for WorkspacePanelLayoutDto {
    fn from(value: WorkspacePanelLayout) -> Self {
        Self {
            left_width: value.left_width,
            right_width: value.right_width,
        }
    }
}

impl From<ThemePreferenceDto> for ThemePreference {
    fn from(value: ThemePreferenceDto) -> Self {
        match value {
            ThemePreferenceDto::System => Self::System,
            ThemePreferenceDto::Light => Self::Light,
            ThemePreferenceDto::Dark => Self::Dark,
        }
    }
}

impl From<ThemePreference> for ThemePreferenceDto {
    fn from(value: ThemePreference) -> Self {
        match value {
            ThemePreference::System => Self::System,
            ThemePreference::Light => Self::Light,
            ThemePreference::Dark => Self::Dark,
        }
    }
}

impl From<StartupBehaviorDto> for StartupBehavior {
    fn from(value: StartupBehaviorDto) -> Self {
        match value {
            StartupBehaviorDto::RestoreWorkspace => Self::RestoreWorkspace,
            StartupBehaviorDto::NewMarkdown => Self::NewMarkdown,
            StartupBehaviorDto::StartPage => Self::StartPage,
        }
    }
}

impl From<StartupBehavior> for StartupBehaviorDto {
    fn from(value: StartupBehavior) -> Self {
        match value {
            StartupBehavior::RestoreWorkspace => Self::RestoreWorkspace,
            StartupBehavior::NewMarkdown => Self::NewMarkdown,
            StartupBehavior::StartPage => Self::StartPage,
        }
    }
}
