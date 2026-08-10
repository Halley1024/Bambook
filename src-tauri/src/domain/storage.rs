use std::path::{Path, PathBuf};

use crate::domain::document::DocumentKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StorageLayout {
    pub(crate) root: PathBuf,
    pub(crate) storage: PathBuf,
    pub(crate) themes: PathBuf,
    pub(crate) settings_file: PathBuf,
    pub(crate) recent_documents_file: PathBuf,
    pub(crate) closed_documents_file: PathBuf,
    pub(crate) workspace_state_file: PathBuf,
    pub(crate) document_library_file: PathBuf,
}

impl StorageLayout {
    pub(crate) fn required_directories(&self) -> [&Path; 3] {
        [&self.root, &self.storage, &self.themes]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedPdfDocument {
    pub(crate) document_id: String,
    pub(crate) display_name: String,
    pub(crate) source_path: String,
    pub(crate) storage_path: String,
    pub(crate) data_path: String,
    pub(crate) storage_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedMarkdownDocument {
    pub(crate) document_id: String,
    pub(crate) display_name: String,
    pub(crate) source_path: String,
    pub(crate) storage_path: String,
    pub(crate) data_path: String,
    pub(crate) storage_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredDocumentEntry {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) kind: DocumentKind,
    pub(crate) path: String,
    pub(crate) source_path: String,
    pub(crate) data_path: String,
    pub(crate) storage_mode: String,
    pub(crate) created_at: u64,
    pub(crate) updated_at: u64,
    pub(crate) last_opened_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfBookmark {
    pub(crate) id: String,
    pub(crate) document_id: String,
    pub(crate) page: u32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) title: String,
    pub(crate) color: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfReadingState {
    pub(crate) page: u32,
    pub(crate) page_y: Option<f64>,
    pub(crate) zoom_mode: String,
    pub(crate) scale: f64,
    pub(crate) updated_at: String,
}
