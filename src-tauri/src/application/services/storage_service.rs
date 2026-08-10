use std::sync::Arc;

use crate::{
    application::ports::StorageRepository,
    domain::storage::{
        ManagedMarkdownDocument, ManagedPdfDocument, PdfBookmark, PdfReadingState, StorageLayout, StoredDocumentEntry,
    },
    error::AppResult,
};

pub(crate) struct StorageService {
    repository: Arc<dyn StorageRepository>,
}

impl StorageService {
    pub(crate) fn new(repository: Arc<dyn StorageRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn initialize(&self) -> AppResult<StorageLayout> {
        self.repository.initialize()
    }

    pub(crate) fn prepare_pdf(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedPdfDocument> {
        self.repository.prepare_pdf(path, settings)
    }
    pub(crate) fn persist_pdf(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedPdfDocument> {
        self.repository.persist_pdf(path, settings)
    }

    pub(crate) fn prepare_markdown(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedMarkdownDocument> {
        self.repository.prepare_markdown(path, settings)
    }
    pub(crate) fn persist_markdown(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedMarkdownDocument> {
        self.repository.persist_markdown(path, settings)
    }

    pub(crate) fn reconcile_document_storage(
        &self,
        document_id: &str,
        settings: &crate::domain::settings::AppSettings,
    ) -> AppResult<()> {
        self.repository.reconcile_document_storage(document_id, settings)
    }

    pub(crate) fn load_document_library(&self) -> AppResult<Vec<StoredDocumentEntry>> {
        self.repository.load_document_library()
    }

    pub(crate) fn relink_document(&self, document_id: &str, new_path: &str) -> AppResult<StoredDocumentEntry> {
        self.repository.relink_document(document_id, new_path)
    }

    pub(crate) fn export_document_package(&self, document_id: &str, destination: &str) -> AppResult<String> {
        self.repository.export_document_package(document_id, destination)
    }

    pub(crate) fn load_pdf_bookmarks(&self, document_id: &str, source_path: Option<&str>) -> AppResult<Vec<PdfBookmark>> {
        self.repository.load_pdf_bookmarks(document_id, source_path)
    }

    pub(crate) fn save_pdf_bookmarks(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        bookmarks: &[PdfBookmark],
    ) -> AppResult<()> {
        self.repository.save_pdf_bookmarks(document_id, source_path, bookmarks)
    }

    pub(crate) fn load_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
    ) -> AppResult<Option<PdfReadingState>> {
        self.repository.load_pdf_reading_state(document_id, source_path)
    }

    pub(crate) fn save_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        state: &PdfReadingState,
    ) -> AppResult<()> {
        self.repository.save_pdf_reading_state(document_id, source_path, state)
    }
}
