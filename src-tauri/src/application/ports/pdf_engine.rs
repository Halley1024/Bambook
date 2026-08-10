use crate::{
    domain::pdf::{PdfDocumentModel, PdfPageStructure, RenderedPdfPage},
    error::AppResult,
};

pub(crate) trait PdfEngine: Send + Sync {
    fn open(&self, path: &str, password: Option<&str>) -> AppResult<PdfDocumentModel>;
    fn authenticate(&self, session_id: &str, password: &str) -> AppResult<PdfDocumentModel>;
    fn page_structure(&self, session_id: &str, page_index: u32) -> AppResult<PdfPageStructure>;
    fn render_page(
        &self,
        session_id: &str,
        page_index: u32,
        scale: f32,
    ) -> AppResult<RenderedPdfPage>;
    fn close(&self, session_id: &str) -> AppResult<()>;
}
