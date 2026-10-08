use crate::{domain::markdown_ast::MarkdownDocumentModel, error::AppResult};

pub(crate) trait MarkdownSessionRepository: Send + Sync {
    fn insert(&self, document: MarkdownDocumentModel) -> AppResult<()>;
    fn get(&self, session_id: &str) -> AppResult<MarkdownDocumentModel>;
    fn replace(&self, document: MarkdownDocumentModel) -> AppResult<()>;
    fn remove(&self, session_id: &str) -> AppResult<()>;
    fn all_titles(&self) -> Vec<String>;
}
