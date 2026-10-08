/// 应用状态程序
/// 干什么的？
/// 保存的是什么内容？
/// 

use std::{path::PathBuf, sync::Arc};

use crate::{
    application::{
        ports::DocumentRepository,
        services::{
            AnnotationService, DocumentHistoryService, DocumentSearchService, DocumentService,
            ExportService, LlmService, MarkdownDocumentService, McpService, PdfDocumentService,
            SettingsService, StorageService, TranslationService, WorkspaceStateService,
        },
    },
    infrastructure::{
        exporters::MarkdownAnnotationExporter,
        filesystem::{FileDocumentPathInspector, FileDocumentRepository},
        llm::OpenAiCompatibleLlmClient,
        markdown::{PulldownMarkdownExporter, PulldownMarkdownParser},
        mcp::NoopMcpToolRegistry,
        paths::AppPaths,
        pdf::{MupdfPdfEngine, MupdfPdfSearchEngine},
        repositories::{
            JsonAnnotationRepository, JsonDocumentHistoryRepository, JsonSettingsRepository,
            JsonWorkspaceStateRepository, LocalStorageRepository,
        },
        sessions::InMemoryMarkdownSessions,
        translation::BaiduTranslationProvider,
    },
};

pub(crate) struct AppState {
    pub(crate) storage: Arc<StorageService>,
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

impl AppState {
    pub(crate) fn new(data_dir: PathBuf) -> crate::error::AppResult<Self> {
        let paths = AppPaths::new(data_dir);    // 应用的系统标准路径
        let storage = Arc::new(StorageService::new(Arc::new(LocalStorageRepository::new(
            paths.clone(),
        ))));
        storage.initialize()?;
        let document_repository: Arc<dyn DocumentRepository> =
            Arc::new(FileDocumentRepository::new(paths.clone()));
        let markdown_sessions = Arc::new(InMemoryMarkdownSessions::default());
        let pdf_search = Arc::new(MupdfPdfSearchEngine::default());
        Ok(Self {
            storage: Arc::clone(&storage),
            documents: Arc::new(DocumentService::new(Arc::clone(&document_repository))),
            settings: Arc::new(SettingsService::new(Arc::new(JsonSettingsRepository::new(
                paths.clone(),
            )))),
            document_history: Arc::new(DocumentHistoryService::new(
                Arc::new(JsonDocumentHistoryRepository::new(paths.clone())),
                Arc::new(FileDocumentPathInspector),
            )),
            workspace_state: Arc::new(WorkspaceStateService::new(Arc::new(
                JsonWorkspaceStateRepository::new(paths.clone()),
            ))),
            annotations: Arc::new(AnnotationService::new(Arc::new(
                JsonAnnotationRepository::new(paths.clone()),
            ))),
            exports: Arc::new(ExportService::new(Arc::new(
                MarkdownAnnotationExporter::new(paths),
            ))),
            markdown_documents: Arc::new(MarkdownDocumentService::new(
                document_repository,
                Arc::new(PulldownMarkdownParser),
                Arc::new(PulldownMarkdownExporter),
                markdown_sessions.clone(),
            )),
            pdf_documents: Arc::new(PdfDocumentService::new(
                Arc::new(MupdfPdfEngine::default()),
                Arc::clone(&storage),
            )),
            search: Arc::new(DocumentSearchService::new(pdf_search, markdown_sessions)),
            translation: Arc::new(TranslationService::new(vec![Arc::new(
                BaiduTranslationProvider::new()?,
            )])),
            llm: Arc::new(LlmService::new(Arc::new(OpenAiCompatibleLlmClient::new()?))),
            mcp: Arc::new(McpService::new(Arc::new(NoopMcpToolRegistry))),
        })
    }
}
