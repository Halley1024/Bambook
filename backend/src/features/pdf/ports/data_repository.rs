use crate::features::pdf::PdfBookmark;
use crate::features::pdf::PdfReadingState;
use crate::shared::error::AppResult;

pub(crate) trait PdfDataRepository: Send + Sync {
    fn load_pdf_bookmarks(
        &self,
        document_id: &str,
        source_path: Option<&str>,
    ) -> AppResult<Vec<PdfBookmark>>;
    fn save_pdf_bookmarks(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        bookmarks: &[PdfBookmark],
    ) -> AppResult<()>;
    fn load_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
    ) -> AppResult<Option<PdfReadingState>>;
    fn save_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        state: &PdfReadingState,
    ) -> AppResult<()>;
}
