//! Construct each shared repository/session store once, then inject it into feature services.

use std::{path::PathBuf, sync::Arc};

use crate::features::annotations::AnnotationService;
use crate::features::annotations::ExportService;
use crate::features::annotations::JsonAnnotationRepository;
use crate::features::annotations::MarkdownAnnotationExporter;
use crate::features::assistant::llm::LlmService;
use crate::features::assistant::llm::OpenAiCompatibleLlmClient;
use crate::features::assistant::mcp::McpService;
use crate::features::assistant::mcp::NoopMcpToolRegistry;
use crate::features::assistant::translation::BaiduTranslationProvider;
use crate::features::assistant::translation::TranslationService;
use crate::features::documents::DocumentRepository;
use crate::features::documents::DocumentService;
use crate::features::documents::FileDocumentRepository;
use crate::features::history::DocumentHistoryService;
use crate::features::history::FileDocumentPathInspector;
use crate::features::history::JsonDocumentHistoryRepository;
use crate::features::library::LibraryPaths;
use crate::features::library::LibraryService;
use crate::features::library::LocalLibraryRepository;
use crate::features::markdown::InMemoryMarkdownSessions;
use crate::features::markdown::MarkdownDocumentService;
use crate::features::markdown::PulldownMarkdownExporter;
use crate::features::markdown::PulldownMarkdownParser;
use crate::features::pdf::MupdfPdfEngine;
use crate::features::pdf::MupdfPdfSearchEngine;
use crate::features::pdf::PdfDocumentService;
use crate::features::search::DocumentSearchService;
use crate::features::settings::JsonSettingsRepository;
use crate::features::settings::SettingsService;
use crate::features::workspace::JsonWorkspaceStateRepository;
use crate::features::workspace::WorkspaceStateService;

use super::state::AppState;

impl AppState {
    pub(crate) fn new(data_dir: PathBuf) -> crate::shared::error::AppResult<Self> {
        let base_paths = crate::platform::paths::AppPaths::new(data_dir.clone());
        let paths = LibraryPaths::new(data_dir); // 应用的系统标准路径
        let local_storage = Arc::new(LocalLibraryRepository::new(paths.clone()));
        let storage = Arc::new(LibraryService::new(local_storage.clone()));
        storage.initialize()?;
        let document_repository: Arc<dyn DocumentRepository> = Arc::new(FileDocumentRepository);
        let markdown_repository = Arc::new(crate::features::markdown::FileMarkdownRepository::new(
            base_paths.clone(),
        ));
        let markdown_sessions = Arc::new(InMemoryMarkdownSessions::default());
        let pdf_search = Arc::new(MupdfPdfSearchEngine::default());
        Ok(Self {
            pdf_data: Arc::new(crate::features::pdf::PdfPersistenceService::new(Arc::new(
                crate::features::pdf::JsonPdfDataRepository::new(local_storage),
            ))),
            library: Arc::clone(&storage),
            documents: Arc::new(DocumentService::new(Arc::clone(&document_repository))),
            settings: Arc::new(SettingsService::new(Arc::new(JsonSettingsRepository::new(
                base_paths.clone(),
            )))),
            document_history: Arc::new(DocumentHistoryService::new(
                Arc::new(JsonDocumentHistoryRepository::new(base_paths.clone())),
                Arc::new(FileDocumentPathInspector),
            )),
            workspace_state: Arc::new(WorkspaceStateService::new(Arc::new(
                JsonWorkspaceStateRepository::new(base_paths.clone()),
            ))),
            annotations: Arc::new(AnnotationService::new(Arc::new(
                JsonAnnotationRepository::new(paths.clone()),
            ))),
            exports: Arc::new(ExportService::new(Arc::new(
                MarkdownAnnotationExporter::new(paths),
            ))),
            markdown_documents: Arc::new(MarkdownDocumentService::new(
                markdown_repository,
                Arc::new(PulldownMarkdownParser),
                Arc::new(PulldownMarkdownExporter),
                markdown_sessions.clone(),
            )),
            pdf_documents: Arc::new(PdfDocumentService::new(
                Arc::new(MupdfPdfEngine::default()),
                Arc::clone(&storage),
            )),
            search: Arc::new(DocumentSearchService::new(
                pdf_search,
                Arc::new(crate::features::markdown::MarkdownSearchAdapter::new(
                    markdown_sessions,
                )),
            )),
            translation: Arc::new(TranslationService::new(vec![Arc::new(
                BaiduTranslationProvider::new()?,
            )])),
            llm: Arc::new(LlmService::new(Arc::new(OpenAiCompatibleLlmClient::new()?))),
            mcp: Arc::new(McpService::new(Arc::new(NoopMcpToolRegistry))),
        })
    }
}

impl AppState {
    pub(crate) fn register(self, app: &tauri::App) {
        use tauri::Manager;
        app.manage(Arc::clone(&self.documents));
        app.manage(Arc::clone(&self.pdf_documents));
        app.manage(Arc::clone(&self.pdf_data));
        app.manage(Arc::clone(&self.markdown_documents));
        app.manage(Arc::clone(&self.annotations));
        app.manage(Arc::clone(&self.exports));
        app.manage(Arc::clone(&self.document_history));
        app.manage(Arc::clone(&self.workspace_state));
        app.manage(Arc::clone(&self.library));
        app.manage(Arc::clone(&self.search));
        app.manage(Arc::clone(&self.settings));
        app.manage(Arc::clone(&self.llm));
        app.manage(Arc::clone(&self.translation));
        app.manage(Arc::clone(&self.mcp));
        app.manage(self);
    }
}
