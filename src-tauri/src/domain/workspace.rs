use crate::domain::document::DocumentKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceDocument {
    pub(crate) path: String,
    pub(crate) kind: DocumentKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceState {
    pub(crate) open_documents: Vec<WorkspaceDocument>,
    pub(crate) active_document_path: Option<String>,
    pub(crate) folder_path: Option<String>,
}
