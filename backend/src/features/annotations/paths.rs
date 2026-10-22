use crate::features::library::{validate_document_id, LibraryPaths};
use crate::features::settings::AppSettings;
use crate::shared::error::AppResult;
use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct AnnotationPaths {
    library: LibraryPaths,
}
impl AnnotationPaths {
    pub(crate) fn new(library: LibraryPaths) -> Self {
        Self { library }
    }
    pub(crate) fn annotation_file(
        &self,
        document_id: &str,
        settings: &AppSettings,
    ) -> AppResult<PathBuf> {
        validate_document_id(document_id)?;
        let custom_path = settings
            .annotation_storage_dir
            .as_deref()
            .map(PathBuf::from)
            .map(|directory| directory.join(format!("{document_id}.json")));
        Ok(custom_path.unwrap_or_else(|| {
            self.library
                .indexed_document_directory(document_id)
                .unwrap_or_else(|| self.library.pdf_directory_for(document_id, settings))
                .join("annotations.json")
        }))
    }

    pub(crate) fn legacy_annotation_file(&self, document_id: &str) -> AppResult<PathBuf> {
        validate_document_id(document_id)?;
        Ok(self
            .library
            .storage_layout()
            .storage
            .join("annotations-json")
            .join(format!("{document_id}.json")))
    }
}
