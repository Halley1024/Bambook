use crate::features::search::ports::{
    markdown_search::MarkdownSearchProvider, pdf_search_engine::PdfSearchEngine,
};
use crate::features::search::{SearchRequest, SearchResponse, SearchTarget};
use crate::shared::error::AppResult;
use std::sync::Arc;

pub(crate) struct DocumentSearchService {
    pdf: Arc<dyn PdfSearchEngine>,
    markdown: Arc<dyn MarkdownSearchProvider>,
}
impl DocumentSearchService {
    pub(crate) fn new(
        pdf: Arc<dyn PdfSearchEngine>,
        markdown: Arc<dyn MarkdownSearchProvider>,
    ) -> Self {
        Self { pdf, markdown }
    }
    pub(crate) fn register_pdf(
        &self,
        session_id: &str,
        path: &str,
        password: Option<&str>,
    ) -> AppResult<()> {
        self.pdf.register(session_id, path, password)
    }
    pub(crate) fn authenticate_pdf(&self, session_id: &str, password: &str) -> AppResult<()> {
        self.pdf.authenticate(session_id, password)
    }
    pub(crate) fn close_pdf(&self, session_id: &str) -> AppResult<()> {
        self.pdf.close(session_id)
    }
    pub(crate) fn search(&self, request: SearchRequest) -> AppResult<SearchResponse> {
        match &request.target {
            SearchTarget::Pdf { session_id } => self.pdf.search(
                session_id,
                request.request_id,
                &request.query,
                &request.options,
            ),
            SearchTarget::Markdown { .. } => self.markdown.search(&request),
        }
    }
    pub(crate) fn cancel(&self, target: &SearchTarget, request_id: u64) -> AppResult<()> {
        match target {
            SearchTarget::Pdf { session_id } => self.pdf.cancel(session_id, request_id),
            SearchTarget::Markdown { session_id, .. } => {
                self.markdown.cancel(session_id, request_id)
            }
        }
    }
    pub(crate) fn close_markdown(&self, session_id: &str) {
        self.markdown.close(session_id);
    }
}
