use serde::{Deserialize, Serialize};

use crate::{
    application::ports::WorkspaceStateRepository,
    domain::{
        document::DocumentKind,
        workspace::{WorkspaceDocument, WorkspaceState},
    },
    error::AppResult,
    infrastructure::{
        filesystem::{read_optional_json, write_pretty_json},
        paths::AppPaths,
    },
};

const SCHEMA_VERSION: u32 = 1;

pub(crate) struct JsonWorkspaceStateRepository {
    paths: AppPaths,
}

impl JsonWorkspaceStateRepository {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
}

impl WorkspaceStateRepository for JsonWorkspaceStateRepository {
    fn load(&self) -> AppResult<WorkspaceState> {
        let stored: Option<StoredWorkspaceState> =
            read_optional_json(&self.paths.storage_layout().workspace_state_file)?;
        Ok(stored.map(Into::into).unwrap_or_default())
    }

    fn save(&self, state: &WorkspaceState) -> AppResult<()> {
        write_pretty_json(
            &self.paths.storage_layout().workspace_state_file,
            &StoredWorkspaceState::from(state),
        )
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredWorkspaceState {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    open_documents: Vec<StoredWorkspaceDocument>,
    #[serde(default)]
    active_document_path: Option<String>,
    #[serde(default)]
    folder_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredWorkspaceDocument {
    path: String,
    kind: StoredDocumentKind,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum StoredDocumentKind {
    Pdf,
    Markdown,
}

impl From<StoredWorkspaceState> for WorkspaceState {
    fn from(value: StoredWorkspaceState) -> Self {
        Self {
            open_documents: value.open_documents.into_iter().map(Into::into).collect(),
            active_document_path: value.active_document_path,
            folder_path: value.folder_path,
        }
    }
}

impl From<&WorkspaceState> for StoredWorkspaceState {
    fn from(value: &WorkspaceState) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            open_documents: value.open_documents.iter().map(Into::into).collect(),
            active_document_path: value.active_document_path.clone(),
            folder_path: value.folder_path.clone(),
        }
    }
}

impl From<StoredWorkspaceDocument> for WorkspaceDocument {
    fn from(value: StoredWorkspaceDocument) -> Self {
        Self {
            path: value.path,
            kind: match value.kind {
                StoredDocumentKind::Pdf => DocumentKind::Pdf,
                StoredDocumentKind::Markdown => DocumentKind::Markdown,
            },
        }
    }
}

impl From<&WorkspaceDocument> for StoredWorkspaceDocument {
    fn from(value: &WorkspaceDocument) -> Self {
        Self {
            path: value.path.clone(),
            kind: match value.kind {
                DocumentKind::Pdf => StoredDocumentKind::Pdf,
                DocumentKind::Markdown => StoredDocumentKind::Markdown,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        application::ports::WorkspaceStateRepository,
        domain::{
            document::DocumentKind,
            workspace::{WorkspaceDocument, WorkspaceState},
        },
        infrastructure::paths::AppPaths,
    };

    use super::JsonWorkspaceStateRepository;

    #[test]
    fn workspace_state_round_trips_through_versioned_json() {
        let root = test_root();
        let repository = JsonWorkspaceStateRepository::new(AppPaths::new(root.clone()));
        let state = WorkspaceState {
            open_documents: vec![WorkspaceDocument {
                path: "C:\\docs\\notes.md".into(),
                kind: DocumentKind::Markdown,
            }],
            active_document_path: Some("C:\\docs\\notes.md".into()),
            folder_path: Some("C:\\docs".into()),
        };

        repository.save(&state).unwrap();
        assert_eq!(repository.load().unwrap(), state);
        let json = fs::read_to_string(root.join("workspace-state.json")).unwrap();
        assert!(json.contains("\"schemaVersion\": 1"));
        fs::remove_dir_all(root).unwrap();
    }

    fn test_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "bambook-workspace-state-{}-{}",
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
