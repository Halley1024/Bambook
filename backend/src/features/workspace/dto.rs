use crate::features::documents::PersistedDocumentKindDto;
use crate::features::workspace::model::{WorkspaceDocument, WorkspaceState};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceStateDto {
    pub(crate) open_documents: Vec<WorkspaceDocumentDto>,
    pub(crate) active_document_path: Option<String>,
    pub(crate) folder_path: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkspaceDocumentDto {
    path: String,
    kind: PersistedDocumentKindDto,
}
impl From<WorkspaceState> for WorkspaceStateDto {
    fn from(value: WorkspaceState) -> Self {
        Self {
            open_documents: value.open_documents.into_iter().map(Into::into).collect(),
            active_document_path: value.active_document_path,
            folder_path: value.folder_path,
        }
    }
}
impl From<WorkspaceStateDto> for WorkspaceState {
    fn from(value: WorkspaceStateDto) -> Self {
        Self {
            open_documents: value.open_documents.into_iter().map(Into::into).collect(),
            active_document_path: value.active_document_path,
            folder_path: value.folder_path,
        }
    }
}
impl From<WorkspaceDocument> for WorkspaceDocumentDto {
    fn from(value: WorkspaceDocument) -> Self {
        Self {
            path: value.path,
            kind: value.kind.into(),
        }
    }
}
impl From<WorkspaceDocumentDto> for WorkspaceDocument {
    fn from(value: WorkspaceDocumentDto) -> Self {
        Self {
            path: value.path,
            kind: value.kind.into(),
        }
    }
}
