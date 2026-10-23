use std::fs;
use std::path::PathBuf;

use crate::features::library::StorageLayout;
use crate::features::settings::AppSettings;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[derive(Debug, Clone)]
pub(crate) struct LibraryPaths {
    base: crate::platform::paths::AppPaths,
}

impl LibraryPaths {
    pub(crate) fn new(data_dir: PathBuf) -> Self {
        Self {
            base: crate::platform::paths::AppPaths::new(data_dir),
        }
    }

    pub(crate) fn settings_file(&self) -> PathBuf {
        self.base.root().join("settings.json")
    }

    pub(crate) fn storage_layout(&self) -> StorageLayout {
        let storage = self.base.root().join("storage");
        StorageLayout {
            root: self.base.root().to_path_buf(),
            themes: self.base.root().join("themes"),
            settings_file: self.settings_file(),
            recent_documents_file: self.base.root().join("recent-documents.json"),
            closed_documents_file: self.base.root().join("closed-documents.json"),
            workspace_state_file: self.base.root().join("workspace-state.json"),
            document_library_file: self.base.root().join("document-library.json"),
            storage,
        }
    }

    pub(crate) fn pdf_directory(&self, document_id: &str) -> PathBuf {
        self.storage_layout().storage.join(document_id)
    }

    pub(crate) fn indexed_document_directory(&self, document_id: &str) -> Option<PathBuf> {
        let content = fs::read_to_string(self.storage_layout().document_library_file).ok()?;
        let value: serde_json::Value = serde_json::from_str(&content).ok()?;
        value
            .get("documents")?
            .as_array()?
            .iter()
            .find_map(|entry| {
                (entry.get("id")?.as_str()? == document_id)
                    .then(|| entry.get("dataPath")?.as_str())
                    .flatten()
            })
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
    }

    pub(crate) fn pdf_directory_for(&self, document_id: &str, settings: &AppSettings) -> PathBuf {
        settings
            .pdf_storage_dir
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.storage_layout().storage)
            .join(document_id)
    }

    pub(crate) fn markdown_directory_for(
        &self,
        document_id: &str,
        settings: &AppSettings,
    ) -> PathBuf {
        settings
            .markdown_storage_dir
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.storage_layout().storage)
            .join(document_id)
    }

    pub(crate) fn pdf_document_file_for(
        &self,
        document_id: &str,
        settings: &AppSettings,
    ) -> PathBuf {
        self.pdf_directory_for(document_id, settings)
            .join("document.pdf")
    }

    pub(crate) fn pdf_metadata_file_for(
        &self,
        document_id: &str,
        settings: &AppSettings,
    ) -> PathBuf {
        self.pdf_directory_for(document_id, settings)
            .join("metadata.json")
    }

    pub(crate) fn pdf_document_file(&self, document_id: &str) -> PathBuf {
        self.pdf_directory(document_id).join("document.pdf")
    }

    pub(crate) fn pdf_metadata_file(&self, document_id: &str) -> PathBuf {
        self.pdf_directory(document_id).join("metadata.json")
    }

    pub(crate) fn export_dir(&self, settings: &AppSettings) -> PathBuf {
        settings
            .export_dir
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.storage_layout().storage.join("exports"))
    }
}

pub(crate) fn validate_document_id(value: &str) -> AppResult<()> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(AppError::InvalidDocumentId)
    }
}

pub(crate) fn path_to_user_string(path: &std::path::Path) -> String {
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = value.strip_prefix(r"\\?\") {
        rest.to_owned()
    } else {
        value.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{path_to_user_string, validate_document_id, LibraryPaths};

    #[test]
    fn accepts_fingerprints_and_rejects_path_segments() {
        assert!(validate_document_id("a9f0-_reader_1").is_ok());
        assert!(validate_document_id("../settings").is_err());
        assert!(validate_document_id("folder\\file").is_err());
        assert!(validate_document_id("").is_err());
    }

    #[test]
    fn derives_the_complete_bambook_storage_layout_from_one_root() {
        let paths = LibraryPaths::new(PathBuf::from("Bambook"));
        let layout = paths.storage_layout();

        assert_eq!(layout.storage, PathBuf::from("Bambook/storage"));
        assert_eq!(layout.themes, PathBuf::from("Bambook/themes"));
        assert_eq!(layout.settings_file, PathBuf::from("Bambook/settings.json"));
        assert_eq!(
            paths.pdf_document_file("pdf-123"),
            PathBuf::from("Bambook/storage/pdf-123/document.pdf")
        );
    }

    #[test]
    fn removes_windows_verbatim_prefixes_from_user_facing_paths() {
        assert_eq!(
            path_to_user_string(std::path::Path::new(r"\\?\C:\Docs\book.pdf")),
            r"C:\Docs\book.pdf"
        );
        assert_eq!(
            path_to_user_string(std::path::Path::new(r"\\?\UNC\server\share\book.pdf")),
            r"\\server\share\book.pdf"
        );
    }
}
