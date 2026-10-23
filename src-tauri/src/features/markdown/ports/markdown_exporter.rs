use crate::features::markdown::MarkdownDocumentModel;
use crate::shared::error::AppResult;

pub(crate) trait MarkdownExporter: Send + Sync {
    fn export_html(&self, document: &MarkdownDocumentModel, output_path: &str) -> AppResult<()>;
    fn export_pdf(&self, document: &MarkdownDocumentModel, output_path: &str) -> AppResult<()>;
}
