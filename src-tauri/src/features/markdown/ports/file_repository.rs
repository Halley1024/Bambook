use crate::features::documents::OpenedDocument;
use crate::shared::error::AppResult;

pub(crate) trait MarkdownFileRepository: Send + Sync {
    fn create_markdown(&self, path: &str) -> AppResult<()>;
    fn read_text(&self, path: &str) -> AppResult<OpenedDocument>;
    fn save_markdown(&self, path: &str, text: &str) -> AppResult<()>;
    fn save_untitled_recovery(&self, content: &str) -> AppResult<()>;
    fn load_untitled_recovery(&self) -> AppResult<Option<String>>;
    fn clear_untitled_recovery(&self) -> AppResult<()>;
}
