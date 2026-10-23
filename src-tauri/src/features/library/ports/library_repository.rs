use crate::features::library::ManagedMarkdownDocument;
use crate::features::library::ManagedPdfDocument;
use crate::features::library::StorageLayout;
use crate::features::library::StoredDocumentEntry;
use crate::shared::error::AppResult;

pub(crate) trait LibraryRepository: Send + Sync {
    fn initialize(&self) -> AppResult<StorageLayout>;
    fn layout(&self) -> StorageLayout;
    fn prepare_pdf(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedPdfDocument>;
    fn persist_pdf(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedPdfDocument>;
    fn prepare_markdown(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedMarkdownDocument>;
    fn persist_markdown(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedMarkdownDocument>;
    fn reconcile_document_storage(
        &self,
        document_id: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<()>;
    fn load_document_library(&self) -> AppResult<Vec<StoredDocumentEntry>>;
    fn relink_document(&self, document_id: &str, new_path: &str) -> AppResult<StoredDocumentEntry>;
    fn export_document_package(&self, document_id: &str, destination: &str) -> AppResult<String>;
}
