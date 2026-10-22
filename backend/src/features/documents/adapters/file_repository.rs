use std::{fs, io::Read, path::Path};

use crate::features::documents::DirectoryEntry;
use crate::features::documents::DocumentKind;
use crate::features::documents::DocumentRepository;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

use super::directory_reader::{collect_markdown_entries, collect_supported_entries};

pub(crate) struct FileDocumentRepository;

impl DocumentRepository for FileDocumentRepository {
    // 实现DocumentRepository中重命名接口
    fn rename(&self, path: &str, new_path: &str) -> AppResult<()> {
        fs::rename(path, new_path)?;
        Ok(())
    }
    fn detect_kind(&self, path: &str) -> AppResult<DocumentKind> {
        let path = Path::new(path);
        let mut header = [0_u8; 5];
        let mut file = fs::File::open(path)?;
        let read = file.read(&mut header)?;
        detect_kind(path, &header[..read])
    }

    fn list_markdown_directory(&self, document_path: &str) -> AppResult<Vec<DirectoryEntry>> {
        let directory = Path::new(document_path)
            .parent()
            .ok_or(AppError::MissingParentDirectory)?;
        collect_markdown_entries(directory, 0)
    }

    fn list_supported_directory(&self, directory_path: &str) -> AppResult<Vec<DirectoryEntry>> {
        let directory = Path::new(directory_path);
        if !directory.is_dir() {
            return Err(AppError::MissingParentDirectory);
        }
        collect_supported_entries(directory, 0)
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
