use std::sync::Arc;

use crate::features::documents::DirectoryEntry;
use crate::features::documents::DocumentKind;
use crate::features::documents::DocumentRepository;
use crate::shared::error::AppResult;

pub(crate) struct DocumentService {
    repository: Arc<dyn DocumentRepository>,
}

impl DocumentService {
    pub(crate) fn rename(&self, path: &str, new_path: &str, markdown: bool) -> AppResult<()> {
        if markdown && !new_path.ends_with(".md") {
            return Err(crate::shared::error::AppError::InvalidMarkdownPath);
        }
        self.repository.rename(path, new_path)
    }

    pub(crate) fn new(repository: Arc<dyn DocumentRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn list_markdown_directory(
        &self,
        document_path: &str,
    ) -> AppResult<Vec<DirectoryEntry>> {
        self.repository.list_markdown_directory(document_path)
    }

    pub(crate) fn list_supported_directory(
        &self,
        directory_path: &str,
    ) -> AppResult<Vec<DirectoryEntry>> {
        self.repository.list_supported_directory(directory_path)
    }

    pub(crate) fn detect_kind(&self, path: &str) -> AppResult<DocumentKind> {
        self.repository.detect_kind(path)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::features::documents::DirectoryEntry;
    use crate::features::documents::DocumentRepository;
    use crate::shared::error::AppResult;

    use super::DocumentService;

    struct FakeDocumentRepository;

    impl DocumentRepository for FakeDocumentRepository {
        fn rename(&self, _path: &str, _new_path: &str) -> AppResult<()> {
            Ok(())
        }
        fn detect_kind(&self, _path: &str) -> AppResult<crate::features::documents::DocumentKind> {
            Ok(crate::features::documents::DocumentKind::Markdown)
        }

        fn list_markdown_directory(&self, _document_path: &str) -> AppResult<Vec<DirectoryEntry>> {
            Ok(vec![DirectoryEntry {
                name: "notes.md".into(),
                path: "C:\\docs\\notes.md".into(),
                is_directory: false,
                children: Vec::new(),
            }])
        }

        fn list_supported_directory(&self, directory_path: &str) -> AppResult<Vec<DirectoryEntry>> {
            self.list_markdown_directory(directory_path)
        }
    }

    #[test]
    fn delegates_document_use_cases_through_the_port() {
        let service = DocumentService::new(Arc::new(FakeDocumentRepository));

        assert_eq!(
            service
                .list_markdown_directory("C:\\docs\\notes.md")
                .unwrap()
                .len(),
            1
        );
    }
}
