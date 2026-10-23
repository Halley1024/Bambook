use std::sync::Arc;

use crate::features::library::LibraryRepository;
use crate::features::library::ManagedMarkdownDocument;
use crate::features::library::ManagedPdfDocument;
use crate::features::library::StorageLayout;
use crate::features::library::StoredDocumentEntry;
use crate::shared::error::AppResult;

pub(crate) struct LibraryService {
    repository: Arc<dyn LibraryRepository>,
}

impl LibraryService {
    pub(crate) fn inspect_relink(
        &self,
        document_id: &str,
        new_path: &str,
    ) -> AppResult<crate::features::library::RelinkInspection> {
        let entry = self
            .load_document_library()?
            .into_iter()
            .find(|item| item.id == document_id)
            .ok_or(crate::shared::error::AppError::UnsupportedDocumentType)?;
        let selected_name = std::path::Path::new(new_path)
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(crate::shared::error::AppError::MissingFileName)?
            .to_owned();
        Ok(crate::features::library::RelinkInspection {
            name_matches: entry.title.eq_ignore_ascii_case(&selected_name),
            recorded_name: entry.title,
            selected_name,
        })
    }
    pub(crate) fn new(repository: Arc<dyn LibraryRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn initialize(&self) -> AppResult<StorageLayout> {
        self.repository.initialize()
    }

    pub(crate) fn prepare_pdf(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedPdfDocument> {
        self.repository.prepare_pdf(path, settings)
    }
    pub(crate) fn persist_pdf(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedPdfDocument> {
        self.repository.persist_pdf(path, settings)
    }

    pub(crate) fn prepare_markdown(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedMarkdownDocument> {
        self.repository.prepare_markdown(path, settings)
    }
    pub(crate) fn persist_markdown(
        &self,
        path: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<ManagedMarkdownDocument> {
        self.repository.persist_markdown(path, settings)
    }

    pub(crate) fn reconcile_document_storage(
        &self,
        document_id: &str,
        settings: &crate::features::settings::AppSettings,
    ) -> AppResult<()> {
        self.repository
            .reconcile_document_storage(document_id, settings)
    }

    pub(crate) fn load_document_library(&self) -> AppResult<Vec<StoredDocumentEntry>> {
        self.repository.load_document_library()
    }

    pub(crate) fn relink_document(
        &self,
        document_id: &str,
        new_path: &str,
    ) -> AppResult<StoredDocumentEntry> {
        self.repository.relink_document(document_id, new_path)
    }

    pub(crate) fn export_document_package(
        &self,
        document_id: &str,
        destination: &str,
    ) -> AppResult<String> {
        self.repository
            .export_document_package(document_id, destination)
    }
}
