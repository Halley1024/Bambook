use serde::{Deserialize, Serialize};

use crate::{
    application::ports::DocumentHistoryRepository,
    domain::{
        document::DocumentKind,
        document_history::{DocumentAvailability, DocumentHistoryEntry},
    },
    error::AppResult,
    infrastructure::{
        filesystem::{read_optional_json, write_pretty_json},
        paths::AppPaths,
    },
};

const SCHEMA_VERSION: u32 = 2;

pub(crate) struct JsonDocumentHistoryRepository {
    paths: AppPaths,
}

impl JsonDocumentHistoryRepository {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    fn load_file(&self, path: &std::path::Path) -> AppResult<Vec<DocumentHistoryEntry>> {
        let stored: Option<StoredDocumentList> = read_optional_json(path)?;
        Ok(stored
            .unwrap_or_default()
            .documents
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn save_file(&self, path: &std::path::Path, entries: &[DocumentHistoryEntry]) -> AppResult<()> {
        write_pretty_json(
            path,
            &StoredDocumentList {
                schema_version: SCHEMA_VERSION,
                documents: entries.iter().map(Into::into).collect(),
            },
        )
    }
}

impl DocumentHistoryRepository for JsonDocumentHistoryRepository {
    fn load_recent(&self) -> AppResult<Vec<DocumentHistoryEntry>> {
        self.load_file(&self.paths.storage_layout().recent_documents_file)
    }

    fn save_recent(&self, entries: &[DocumentHistoryEntry]) -> AppResult<()> {
        self.save_file(&self.paths.storage_layout().recent_documents_file, entries)
    }

    fn load_closed(&self) -> AppResult<Vec<DocumentHistoryEntry>> {
        self.load_file(&self.paths.storage_layout().closed_documents_file)
    }

    fn save_closed(&self, entries: &[DocumentHistoryEntry]) -> AppResult<()> {
        self.save_file(&self.paths.storage_layout().closed_documents_file, entries)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredDocumentList {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    documents: Vec<StoredDocumentHistoryEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredDocumentHistoryEntry {
    #[serde(default)]
    document_id: Option<String>,
    path: String,
    title: String,
    kind: StoredDocumentKind,
    last_closed_at: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum StoredDocumentKind {
    Pdf,
    Markdown,
}

impl From<StoredDocumentHistoryEntry> for DocumentHistoryEntry {
    fn from(value: StoredDocumentHistoryEntry) -> Self {
        Self {
            document_id: value.document_id,
            path: value.path,
            title: value.title,
            kind: match value.kind {
                StoredDocumentKind::Pdf => DocumentKind::Pdf,
                StoredDocumentKind::Markdown => DocumentKind::Markdown,
            },
            last_closed_at: value.last_closed_at,
            availability: DocumentAvailability::Unknown,
        }
    }
}

impl From<&DocumentHistoryEntry> for StoredDocumentHistoryEntry {
    fn from(value: &DocumentHistoryEntry) -> Self {
        Self {
            document_id: value.document_id.clone(),
            path: value.path.clone(),
            title: value.title.clone(),
            kind: match value.kind {
                DocumentKind::Pdf => StoredDocumentKind::Pdf,
                DocumentKind::Markdown => StoredDocumentKind::Markdown,
            },
            last_closed_at: value.last_closed_at,
        }
    }
}
