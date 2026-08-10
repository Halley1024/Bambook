use std::sync::Arc;

use crate::{
    application::ports::DocumentRepository,
    domain::{document::DocumentKind, markdown::MarkdownFileEntry},
    error::AppResult,
};

pub(crate) struct DocumentService {
    repository: Arc<dyn DocumentRepository>,
}

impl DocumentService {
    pub(crate) fn new(repository: Arc<dyn DocumentRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn list_markdown_directory(
        &self,
        document_path: &str,
    ) -> AppResult<Vec<MarkdownFileEntry>> {
        self.repository.list_markdown_directory(document_path)
    }

    pub(crate) fn list_supported_directory(
        &self,
        directory_path: &str,
    ) -> AppResult<Vec<MarkdownFileEntry>> {
        self.repository.list_supported_directory(directory_path)
    }

    pub(crate) fn detect_kind(&self, path: &str) -> AppResult<DocumentKind> {
        self.repository.detect_kind(path)
    }

    pub(crate) fn create_markdown(&self, path: &str) -> AppResult<()> {
        self.repository.create_markdown(path)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        application::ports::DocumentRepository,
        domain::{document::OpenedDocument, markdown::MarkdownFileEntry},
        error::{AppError, AppResult},
    };

    use super::DocumentService;

    struct FakeDocumentRepository;

    impl DocumentRepository for FakeDocumentRepository {
        fn detect_kind(&self, _path: &str) -> AppResult<crate::domain::document::DocumentKind> {
            Ok(crate::domain::document::DocumentKind::Markdown)
        }
        fn read_text(&self, path: &str) -> AppResult<OpenedDocument> {
            Ok(OpenedDocument {
                path: path.into(),
                name: "notes.md".into(),
                text: "# Notes".into(),
            })
        }

        fn create_markdown(&self, path: &str) -> AppResult<()> {
            self.save_markdown(path, "")
        }

        fn save_markdown(&self, path: &str, _text: &str) -> AppResult<()> {
            if path.ends_with(".md") {
                Ok(())
            } else {
                Err(AppError::InvalidMarkdownPath)
            }
        }

        fn list_markdown_directory(
            &self,
            _document_path: &str,
        ) -> AppResult<Vec<MarkdownFileEntry>> {
            Ok(vec![MarkdownFileEntry {
                name: "notes.md".into(),
                path: "C:\\docs\\notes.md".into(),
                is_directory: false,
                children: Vec::new(),
            }])
        }

        fn list_supported_directory(
            &self,
            directory_path: &str,
        ) -> AppResult<Vec<MarkdownFileEntry>> {
            self.list_markdown_directory(directory_path)
        }

        fn save_untitled_recovery(&self, _content: &str) -> AppResult<()> {
            Ok(())
        }

        fn load_untitled_recovery(&self) -> AppResult<Option<String>> {
            Ok(None)
        }

        fn clear_untitled_recovery(&self) -> AppResult<()> {
            Ok(())
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
