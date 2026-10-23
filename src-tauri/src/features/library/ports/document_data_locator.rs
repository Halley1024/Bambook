use crate::shared::error::AppResult;
use std::path::PathBuf;

pub(crate) trait DocumentDataLocator: Send + Sync {
    fn persisted_document_data_directory(&self, document_id: &str) -> AppResult<Option<PathBuf>>;
    fn companion_read_path(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        file_name: &str,
    ) -> AppResult<Option<PathBuf>>;
}
