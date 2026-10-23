use std::sync::Arc;

use crate::features::annotations::AnnotationService;
use crate::features::annotations::ExportService;
use crate::features::assistant::llm::LlmService;
use crate::features::assistant::mcp::McpService;
use crate::features::assistant::translation::TranslationService;
use crate::features::documents::DocumentService;
use crate::features::history::DocumentHistoryService;
use crate::features::library::LibraryService;
use crate::features::markdown::MarkdownDocumentService;
use crate::features::pdf::PdfDocumentService;
use crate::features::search::DocumentSearchService;
use crate::features::settings::SettingsService;
use crate::features::workspace::WorkspaceStateService;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) pdf_data: Arc<crate::features::pdf::PdfPersistenceService>,
    pub(crate) library: Arc<LibraryService>,
    pub(crate) documents: Arc<DocumentService>,
    pub(crate) settings: Arc<SettingsService>,
    pub(crate) document_history: Arc<DocumentHistoryService>,
    pub(crate) workspace_state: Arc<WorkspaceStateService>,
    pub(crate) annotations: Arc<AnnotationService>,
    pub(crate) exports: Arc<ExportService>,
    pub(crate) markdown_documents: Arc<MarkdownDocumentService>,
    pub(crate) pdf_documents: Arc<PdfDocumentService>,
    pub(crate) search: Arc<DocumentSearchService>,
    pub(crate) translation: Arc<TranslationService>,
    pub(crate) llm: Arc<LlmService>,
    pub(crate) mcp: Arc<McpService>,
}
