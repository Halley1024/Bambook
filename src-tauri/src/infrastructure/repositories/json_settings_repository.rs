use serde::{Deserialize, Serialize};

use crate::{
    application::ports::SettingsRepository,
    domain::settings::{AppSettings, DocumentImportMode, StartupBehavior, ThemePreference, WorkspacePanelLayout},
    error::AppResult,
    infrastructure::{
        filesystem::{read_optional_json, write_pretty_json},
        paths::AppPaths,
    },
};

pub(crate) struct JsonSettingsRepository {
    paths: AppPaths,
}

impl JsonSettingsRepository {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
}

impl SettingsRepository for JsonSettingsRepository {
    fn load(&self) -> AppResult<AppSettings> {
        let stored: Option<StoredSettings> = read_optional_json(&self.paths.settings_file())?;
        let Some(stored) = stored else {
            return Ok(AppSettings::default());
        };
        let requires_migration = stored.schema_version < SETTINGS_SCHEMA_VERSION;
        let settings: AppSettings = stored.into();
        if requires_migration {
            self.save(&settings)?;
        }
        Ok(settings)
    }

    fn save(&self, settings: &AppSettings) -> AppResult<()> {
        write_pretty_json(&self.paths.settings_file(), &StoredSettings::from(settings))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredSettings {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    annotation_storage_dir: Option<String>,
    #[serde(default)]
    export_dir: Option<String>,
    #[serde(default)]
    pdf_storage_dir: Option<String>,
    #[serde(default)]
    markdown_storage_dir: Option<String>,
    #[serde(default)]
    pdf_import_mode: StoredDocumentImportMode,
    #[serde(default)]
    markdown_import_mode: StoredDocumentImportMode,
    #[serde(default)]
    theme: StoredThemePreference,
    #[serde(default = "default_true")]
    animations_enabled: bool,
    #[serde(default = "default_true")]
    confirmation_reminders_enabled: bool,
    #[serde(default)]
    startup_behavior: StoredStartupBehavior,
    #[serde(default = "default_pdf_workspace_layout")]
    pdf_workspace_layout: StoredWorkspacePanelLayout,
    #[serde(default = "default_markdown_workspace_layout")]
    markdown_workspace_layout: StoredWorkspacePanelLayout,
    #[serde(default = "default_translation_provider")]
    translation_provider: String,
    #[serde(default = "default_translation_api_url")]
    translation_api_url: String,
    #[serde(default)]
    translation_api_key: String,
    #[serde(default)]
    translation_secret_id: String,
    #[serde(default)]
    translation_secret_key: String,
    #[serde(default = "default_translation_target_language")]
    translation_target_language: String,
    #[serde(default)]
    allow_third_party_translation: bool,
    #[serde(default = "default_llm_base_url", alias = "localLlmBaseUrl")]
    llm_base_url: String,
    #[serde(default = "default_llm_model", alias = "localLlmModel")]
    llm_model: String,
    #[serde(default, alias = "localLlmApiKey")]
    llm_api_key: String,
    #[serde(default = "default_llm_timeout_seconds", alias = "localLlmTimeoutSeconds")]
    llm_timeout_seconds: u64,
    #[serde(default = "default_llm_temperature", alias = "localLlmTemperature")]
    llm_temperature: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredWorkspacePanelLayout { left_width: u32, right_width: u32 }

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
enum StoredThemePreference { #[default] System, Light, Dark }

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
enum StoredStartupBehavior { #[default] RestoreWorkspace, NewMarkdown, StartPage }

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
enum StoredDocumentImportMode { #[default] Copy, Link }

fn default_true() -> bool { true }
fn default_translation_provider() -> String { "baidu".into() }
fn default_translation_api_url() -> String { "https://fanyi-api.baidu.com/api/trans/vip/translate".into() }
fn default_translation_target_language() -> String { "zh".into() }
fn default_llm_base_url() -> String { "http://127.0.0.1:8080".into() }
fn default_llm_model() -> String { "local-model".into() }
fn default_llm_timeout_seconds() -> u64 { 120 }
fn default_llm_temperature() -> f32 { 0.2 }

fn default_pdf_workspace_layout() -> StoredWorkspacePanelLayout { WorkspacePanelLayout::pdf_default().into() }
fn default_markdown_workspace_layout() -> StoredWorkspacePanelLayout { WorkspacePanelLayout::markdown_default().into() }

const SETTINGS_SCHEMA_VERSION: u32 = 9;

impl From<StoredSettings> for AppSettings {
    fn from(value: StoredSettings) -> Self {
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

impl From<&AppSettings> for StoredSettings {
    fn from(value: &AppSettings) -> Self {
        Self {
            schema_version: SETTINGS_SCHEMA_VERSION,
            annotation_storage_dir: value.annotation_storage_dir.clone(),
            export_dir: value.export_dir.clone(),
            pdf_storage_dir: value.pdf_storage_dir.clone(),
            markdown_storage_dir: value.markdown_storage_dir.clone(),
            pdf_import_mode: value.pdf_import_mode.into(),
            markdown_import_mode: value.markdown_import_mode.into(),
            theme: value.theme.into(),
            animations_enabled: value.animations_enabled,
            confirmation_reminders_enabled: value.confirmation_reminders_enabled,
            startup_behavior: value.startup_behavior.into(),
            pdf_workspace_layout: value.pdf_workspace_layout.into(),
            markdown_workspace_layout: value.markdown_workspace_layout.into(),
            translation_provider: value.translation_provider.clone(),
            translation_api_url: value.translation_api_url.clone(),
            translation_api_key: value.translation_api_key.clone(),
            translation_secret_id: value.translation_secret_id.clone(),
            translation_secret_key: value.translation_secret_key.clone(),
            translation_target_language: value.translation_target_language.clone(),
            allow_third_party_translation: value.allow_third_party_translation,
            llm_base_url: value.llm_base_url.clone(),
            llm_model: value.llm_model.clone(),
            llm_api_key: value.llm_api_key.clone(),
            llm_timeout_seconds: value.llm_timeout_seconds,
            llm_temperature: value.llm_temperature,
        }
    }
}

impl From<StoredWorkspacePanelLayout> for WorkspacePanelLayout {
    fn from(value: StoredWorkspacePanelLayout) -> Self {
        Self { left_width: value.left_width, right_width: value.right_width }
    }
}

impl From<WorkspacePanelLayout> for StoredWorkspacePanelLayout {
    fn from(value: WorkspacePanelLayout) -> Self {
        Self { left_width: value.left_width, right_width: value.right_width }
    }
}

impl From<StoredDocumentImportMode> for DocumentImportMode {
    fn from(value: StoredDocumentImportMode) -> Self { match value { StoredDocumentImportMode::Copy => Self::Copy, StoredDocumentImportMode::Link => Self::Link } }
}

impl From<DocumentImportMode> for StoredDocumentImportMode {
    fn from(value: DocumentImportMode) -> Self { match value { DocumentImportMode::Copy => Self::Copy, DocumentImportMode::Link => Self::Link } }
}

impl From<StoredThemePreference> for ThemePreference {
    fn from(value: StoredThemePreference) -> Self {
        match value { StoredThemePreference::System => Self::System, StoredThemePreference::Light => Self::Light, StoredThemePreference::Dark => Self::Dark }
    }
}

impl From<ThemePreference> for StoredThemePreference {
    fn from(value: ThemePreference) -> Self {
        match value { ThemePreference::System => Self::System, ThemePreference::Light => Self::Light, ThemePreference::Dark => Self::Dark }
    }
}

impl From<StoredStartupBehavior> for StartupBehavior {
    fn from(value: StoredStartupBehavior) -> Self {
        match value { StoredStartupBehavior::RestoreWorkspace => Self::RestoreWorkspace, StoredStartupBehavior::NewMarkdown => Self::NewMarkdown, StoredStartupBehavior::StartPage => Self::StartPage }
    }
}

impl From<StartupBehavior> for StoredStartupBehavior {
    fn from(value: StartupBehavior) -> Self {
        match value { StartupBehavior::RestoreWorkspace => Self::RestoreWorkspace, StartupBehavior::NewMarkdown => Self::NewMarkdown, StartupBehavior::StartPage => Self::StartPage }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{application::ports::SettingsRepository, infrastructure::paths::AppPaths};

    use super::JsonSettingsRepository;
    use crate::domain::settings::{AppSettings, WorkspacePanelLayout};

    #[test]
    fn loading_legacy_settings_rewrites_them_with_a_schema_version() {
        let root = test_root();
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("settings.json"),
            r#"{"annotationStorageDir":null,"exportDir":null}"#,
        )
        .unwrap();
        let repository = JsonSettingsRepository::new(AppPaths::new(root.clone()));

        repository.load().unwrap();

        let migrated = fs::read_to_string(root.join("settings.json")).unwrap();
        assert!(migrated.contains("\"schemaVersion\": 9"));
        assert!(migrated.contains("\"animationsEnabled\": true"));
        assert!(migrated.contains("\"pdfWorkspaceLayout\""));
        assert!(migrated.contains("\"markdownWorkspaceLayout\""));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_panel_layouts_round_trip_independently() {
        let root = test_root();
        fs::create_dir_all(&root).unwrap();
        let repository = JsonSettingsRepository::new(AppPaths::new(root.clone()));
        let settings = AppSettings {
            pdf_workspace_layout: WorkspacePanelLayout { left_width: 305, right_width: 375 },
            markdown_workspace_layout: WorkspacePanelLayout { left_width: 255, right_width: 325 },
            ..AppSettings::default()
        };

        repository.save(&settings).unwrap();
        let restored = repository.load().unwrap();

        assert_eq!(restored.pdf_workspace_layout, settings.pdf_workspace_layout);
        assert_eq!(restored.markdown_workspace_layout, settings.markdown_workspace_layout);
        fs::remove_dir_all(root).unwrap();
    }

    fn test_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "bambook-settings-migration-{}-{}",
            std::process::id(),
            unique_id()
        ))
    }

    fn unique_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }
}
