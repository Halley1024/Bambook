use crate::{
    domain::{
        document::{DocumentKind, OpenedDocument},
        markdown::MarkdownFileEntry,
    },
    error::AppResult,
};

pub(crate) trait DocumentRepository: Send + Sync {
    fn detect_kind(&self, path: &str) -> AppResult<DocumentKind>;
    fn create_markdown(&self, path: &str) -> AppResult<()>;
    fn read_text(&self, path: &str) -> AppResult<OpenedDocument>;
    fn save_markdown(&self, path: &str, text: &str) -> AppResult<()>;
    fn list_markdown_directory(&self, document_path: &str) -> AppResult<Vec<MarkdownFileEntry>>;
    fn list_supported_directory(&self, directory_path: &str) -> AppResult<Vec<MarkdownFileEntry>>;
    fn save_untitled_recovery(&self, content: &str) -> AppResult<()>;
    fn load_untitled_recovery(&self) -> AppResult<Option<String>>;
    fn clear_untitled_recovery(&self) -> AppResult<()>;
}
