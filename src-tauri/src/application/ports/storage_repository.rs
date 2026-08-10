use crate::{
    domain::storage::{
        ManagedMarkdownDocument, ManagedPdfDocument, PdfBookmark, PdfReadingState, StorageLayout, StoredDocumentEntry,
    },
    error::AppResult,
};

pub(crate) trait StorageRepository: Send + Sync {
    fn initialize(&self) -> AppResult<StorageLayout>;
    fn layout(&self) -> StorageLayout;
    fn prepare_pdf(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedPdfDocument>;
    fn persist_pdf(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedPdfDocument>;
    fn prepare_markdown(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedMarkdownDocument>;
    fn persist_markdown(&self, path: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedMarkdownDocument>;
    fn reconcile_document_storage(&self, document_id: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<()>;
    fn load_document_library(&self) -> AppResult<Vec<StoredDocumentEntry>>;
    fn relink_document(&self, document_id: &str, new_path: &str) -> AppResult<StoredDocumentEntry>;
    fn export_document_package(&self, document_id: &str, destination: &str) -> AppResult<String>;
    fn load_pdf_bookmarks(&self, document_id: &str, source_path: Option<&str>) -> AppResult<Vec<PdfBookmark>>;
    fn save_pdf_bookmarks(&self, document_id: &str, source_path: Option<&str>, bookmarks: &[PdfBookmark]) -> AppResult<()>;
    fn load_pdf_reading_state(&self, document_id: &str, source_path: Option<&str>) -> AppResult<Option<PdfReadingState>>;
    fn save_pdf_reading_state(&self, document_id: &str, source_path: Option<&str>, state: &PdfReadingState) -> AppResult<()>;
}
