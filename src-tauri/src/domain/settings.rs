#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum StartupBehavior {
    #[default]
    RestoreWorkspace,
    NewMarkdown,
    StartPage,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum DocumentImportMode {
    #[default]
    Copy,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkspacePanelLayout {
    pub(crate) left_width: u32,
    pub(crate) right_width: u32,
}

impl WorkspacePanelLayout {
    pub(crate) const fn pdf_default() -> Self {
        Self { left_width: 280, right_width: 340 }
    }

    pub(crate) const fn markdown_default() -> Self {
        Self { left_width: 280, right_width: 310 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppSettings {
    pub(crate) annotation_storage_dir: Option<String>,
    pub(crate) export_dir: Option<String>,
    pub(crate) pdf_storage_dir: Option<String>,
    pub(crate) markdown_storage_dir: Option<String>,
    pub(crate) pdf_import_mode: DocumentImportMode,
    pub(crate) markdown_import_mode: DocumentImportMode,
    pub(crate) theme: ThemePreference,
    pub(crate) animations_enabled: bool,
    pub(crate) confirmation_reminders_enabled: bool,
    pub(crate) startup_behavior: StartupBehavior,
    pub(crate) pdf_workspace_layout: WorkspacePanelLayout,
    pub(crate) markdown_workspace_layout: WorkspacePanelLayout,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            annotation_storage_dir: None,
            export_dir: None,
            pdf_storage_dir: None,
            markdown_storage_dir: None,
            pdf_import_mode: DocumentImportMode::Copy,
            markdown_import_mode: DocumentImportMode::Copy,
            theme: ThemePreference::System,
            animations_enabled: true,
            confirmation_reminders_enabled: true,
            startup_behavior: StartupBehavior::RestoreWorkspace,
            pdf_workspace_layout: WorkspacePanelLayout::pdf_default(),
            markdown_workspace_layout: WorkspacePanelLayout::markdown_default(),
        }
    }
}
