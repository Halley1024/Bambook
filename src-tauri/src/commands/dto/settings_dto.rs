use serde::{Deserialize, Serialize};

use crate::domain::settings::{AppSettings, DocumentImportMode, StartupBehavior, ThemePreference, WorkspacePanelLayout};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DocumentImportModeDto { #[default] Copy, Link }

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
        }
    }
}

impl From<DocumentImportModeDto> for DocumentImportMode {
    fn from(value: DocumentImportModeDto) -> Self { match value { DocumentImportModeDto::Copy => Self::Copy, DocumentImportModeDto::Link => Self::Link } }
}

impl From<DocumentImportMode> for DocumentImportModeDto {
    fn from(value: DocumentImportMode) -> Self { match value { DocumentImportMode::Copy => Self::Copy, DocumentImportMode::Link => Self::Link } }
}

fn default_true() -> bool {
    true
}

fn default_pdf_workspace_layout() -> WorkspacePanelLayoutDto {
    WorkspacePanelLayout::pdf_default().into()
}

fn default_markdown_workspace_layout() -> WorkspacePanelLayoutDto {
    WorkspacePanelLayout::markdown_default().into()
}

impl From<WorkspacePanelLayoutDto> for WorkspacePanelLayout {
    fn from(value: WorkspacePanelLayoutDto) -> Self {
        Self { left_width: value.left_width, right_width: value.right_width }
    }
}

impl From<WorkspacePanelLayout> for WorkspacePanelLayoutDto {
    fn from(value: WorkspacePanelLayout) -> Self {
        Self { left_width: value.left_width, right_width: value.right_width }
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
