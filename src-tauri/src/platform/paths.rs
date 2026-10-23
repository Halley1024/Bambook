use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct AppPaths {
    root: PathBuf,
}
impl AppPaths {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn settings_file(&self) -> PathBuf {
        self.root.join("settings.json")
    }
    pub(crate) fn recent_documents_file(&self) -> PathBuf {
        self.root.join("recent-documents.json")
    }
    pub(crate) fn closed_documents_file(&self) -> PathBuf {
        self.root.join("closed-documents.json")
    }
    pub(crate) fn workspace_state_file(&self) -> PathBuf {
        self.root.join("workspace-state.json")
    }
    pub(crate) fn untitled_recovery_file(&self) -> PathBuf {
        self.root.join("storage/_recovery/untitled.md")
    }
}
