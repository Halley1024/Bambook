use crate::features::pdf::PdfBookmark;
use crate::features::pdf::PdfDataRepository;
use crate::features::pdf::PdfReadingState;
use crate::shared::error::AppResult;
use std::sync::Arc;

pub(crate) struct PdfPersistenceService {
    repository: Arc<dyn PdfDataRepository>,
}
impl PdfPersistenceService {
    pub(crate) fn new(repository: Arc<dyn PdfDataRepository>) -> Self {
        Self { repository }
    }
    pub(crate) fn load_pdf_bookmarks(
        &self,
        document_id: &str,
        source_path: Option<&str>,
    ) -> AppResult<Vec<PdfBookmark>> {
        self.repository.load_pdf_bookmarks(document_id, source_path)
    }

    pub(crate) fn save_pdf_bookmarks(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        bookmarks: &[PdfBookmark],
    ) -> AppResult<()> {
        self.repository
            .save_pdf_bookmarks(document_id, source_path, bookmarks)
    }

    pub(crate) fn load_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
    ) -> AppResult<Option<PdfReadingState>> {
        self.repository
            .load_pdf_reading_state(document_id, source_path)
    }

    pub(crate) fn save_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        state: &PdfReadingState,
    ) -> AppResult<()> {
        self.repository
            .save_pdf_reading_state(document_id, source_path, state)
    }
}
