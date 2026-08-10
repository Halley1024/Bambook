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

fn default_pdf_workspace_layout() -> StoredWorkspacePanelLayout { WorkspacePanelLayout::pdf_default().into() }
fn default_markdown_workspace_layout() -> StoredWorkspacePanelLayout { WorkspacePanelLayout::markdown_default().into() }

const SETTINGS_SCHEMA_VERSION: u32 = 5;

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
        assert!(migrated.contains("\"schemaVersion\": 5"));
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
