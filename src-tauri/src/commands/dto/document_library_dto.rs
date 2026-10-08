use serde::Serialize;

use crate::domain::{document::DocumentKind, storage::StoredDocumentEntry};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredDocumentEntryDto {
    id: String,
    title: String,
    kind: StoredDocumentKindDto,
    path: String,
    source_path: String,
    data_path: String,
    storage_mode: String,
    created_at: u64,
    updated_at: u64,
    last_opened_at: u64,
    path_available: bool,
    source_available: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum StoredDocumentKindDto {
    Pdf,
    Markdown,
}

impl From<StoredDocumentEntry> for StoredDocumentEntryDto {
    fn from(value: StoredDocumentEntry) -> Self {
        let path_available = std::path::Path::new(&value.path).is_file();
        let source_available = std::path::Path::new(&value.source_path).is_file();
        Self {
            id: value.id,
            title: value.title,
            kind: match value.kind {
                DocumentKind::Pdf => StoredDocumentKindDto::Pdf,
                DocumentKind::Markdown => StoredDocumentKindDto::Markdown,
            },
            path: value.path,
            source_path: value.source_path,
            data_path: value.data_path,
            storage_mode: value.storage_mode,
            created_at: value.created_at,
            updated_at: value.updated_at,
            last_opened_at: value.last_opened_at,
            path_available,
            source_available,
        }
    }
}
