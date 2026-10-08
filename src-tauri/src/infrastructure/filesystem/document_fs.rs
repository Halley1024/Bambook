use std::{fs, io::Read, path::Path};

use crate::{
    application::ports::DocumentRepository,
    domain::{
        document::{DocumentKind, OpenedDocument},
        markdown::MarkdownFileEntry,
    },
    error::{AppError, AppResult},
    infrastructure::paths::AppPaths,
};

use super::markdown_file_tree::{collect_markdown_entries, collect_supported_entries, is_markdown_path};

pub(crate) struct FileDocumentRepository {
    paths: AppPaths,
}

impl FileDocumentRepository {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    fn file_name(path: &str) -> AppResult<String> {
        Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or(AppError::MissingFileName)
    }
}

impl DocumentRepository for FileDocumentRepository {
    fn detect_kind(&self, path: &str) -> AppResult<DocumentKind> {
        let path = Path::new(path);
        let mut header = [0_u8; 5];
        let mut file = fs::File::open(path)?;
        let read = file.read(&mut header)?;
        detect_kind(path, &header[..read])
    }

    fn read_text(&self, path: &str) -> AppResult<OpenedDocument> {
        Ok(OpenedDocument {
            name: Self::file_name(path)?,
            path: path.to_string(),
            text: fs::read_to_string(path)?,
        })
    }

    fn create_markdown(&self, path: &str) -> AppResult<()> {
        self.save_markdown(path, "")
    }

    fn save_markdown(&self, path: &str, text: &str) -> AppResult<()> {
        let path = Path::new(path);
        if !is_markdown_path(path) {
            return Err(AppError::InvalidMarkdownPath);
        }
        fs::write(path, text)?;
        Ok(())
    }

    fn list_markdown_directory(&self, document_path: &str) -> AppResult<Vec<MarkdownFileEntry>> {
        let directory = Path::new(document_path)
            .parent()
            .ok_or(AppError::MissingParentDirectory)?;
        collect_markdown_entries(directory, 0)
    }

    fn list_supported_directory(&self, directory_path: &str) -> AppResult<Vec<MarkdownFileEntry>> {
        let directory = Path::new(directory_path);
        if !directory.is_dir() {
            return Err(AppError::MissingParentDirectory);
        }
        collect_supported_entries(directory, 0)
    }

    fn save_untitled_recovery(&self, content: &str) -> AppResult<()> {
        let path = self.paths.untitled_recovery_file();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(())
    }

    fn load_untitled_recovery(&self) -> AppResult<Option<String>> {
        let path = self.paths.untitled_recovery_file();
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        if content.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(content))
    }

    fn clear_untitled_recovery(&self) -> AppResult<()> {
        let path = self.paths.untitled_recovery_file();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

fn detect_kind(path: &Path, header: &[u8]) -> AppResult<DocumentKind> {
    if header.starts_with(b"%PDF-") {
        return Ok(DocumentKind::Pdf);
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if matches!(
        extension.to_ascii_lowercase().as_str(),
        "md" | "markdown" | "mdown" | "mkd"
    ) {
        return Ok(DocumentKind::Markdown);
    }
    Err(AppError::UnsupportedDocumentType)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pdf_by_signature_instead_of_extension() {
        assert_eq!(
            detect_kind(Path::new("renamed.txt"), b"%PDF-1.7").unwrap(),
            DocumentKind::Pdf
        );
    }

    #[test]
    fn accepts_supported_markdown_extensions() {
        assert_eq!(
            detect_kind(Path::new("notes.MARKDOWN"), b"# title").unwrap(),
            DocumentKind::Markdown
        );
        assert!(matches!(
            detect_kind(Path::new("notes.txt"), b"text"),
            Err(AppError::UnsupportedDocumentType)
        ));
    }
}
