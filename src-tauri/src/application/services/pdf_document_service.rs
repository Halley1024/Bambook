use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    application::{ports::PdfEngine, services::StorageService},
    domain::{
        pdf::{PdfDocumentModel, PdfPageStructure, RenderedPdfPage},
        storage::ManagedPdfDocument,
    },
    error::{AppError, AppResult},
};

pub(crate) struct PdfDocumentService {
    engine: Arc<dyn PdfEngine>,
    storage: Arc<StorageService>,
    managed_documents: Mutex<HashMap<String, ManagedPdfDocument>>,
}

impl PdfDocumentService {
    pub(crate) fn new(engine: Arc<dyn PdfEngine>, storage: Arc<StorageService>) -> Self {
        Self {
            engine,
            storage,
            managed_documents: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn open(&self, path: &str, password: Option<&str>, settings: &crate::domain::settings::AppSettings) -> AppResult<PdfDocumentModel> {
        let managed = self.storage.prepare_pdf(path, settings)?;
        let mut document = self.engine.open(&managed.storage_path, password)?;
        apply_managed_identity(&mut document, &managed);
        self.managed_documents
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .insert(document.session_id.clone(), managed);
        Ok(document)
    }

    pub(crate) fn authenticate(
        &self,
        session_id: &str,
        password: &str,
    ) -> AppResult<PdfDocumentModel> {
        let mut document = self.engine.authenticate(session_id, password)?;
        let managed = self
            .managed_documents
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .get(session_id)
            .cloned()
            .ok_or(AppError::PdfSessionNotFound)?;
        apply_managed_identity(&mut document, &managed);
        Ok(document)
    }

    pub(crate) fn page_structure(
        &self,
        session_id: &str,
        page_index: u32,
    ) -> AppResult<PdfPageStructure> {
        self.engine.page_structure(session_id, page_index)
    }

    pub(crate) fn persist(&self, session_id: &str, settings: &crate::domain::settings::AppSettings) -> AppResult<ManagedPdfDocument> {
        let mut staged = self.managed_documents.lock().map_err(|_| AppError::StateUnavailable)?
            .get(session_id).cloned().ok_or(AppError::PdfSessionNotFound)?;
        if let Some(entry) = self.storage.load_document_library()?.into_iter()
            .find(|entry| entry.id == staged.document_id)
        {
            staged.source_path = if std::path::Path::new(&entry.source_path).is_file() {
                entry.source_path
            } else if entry.storage_mode == "managed-copy" && std::path::Path::new(&entry.path).is_file() {
                entry.path
            } else {
                entry.source_path
            };
        }
        let persisted = self.storage.persist_pdf(&staged.source_path, settings)?;
        self.managed_documents.lock().map_err(|_| AppError::StateUnavailable)?
            .insert(session_id.to_owned(), persisted.clone());
        Ok(persisted)
    }

    pub(crate) fn render_page(
        &self,
        session_id: &str,
        page_index: u32,
        scale: f32,
    ) -> AppResult<RenderedPdfPage> {
        if !(0.25..=5.0).contains(&scale) {
            return Err(AppError::PageOutOfRange);
        }
        self.engine.render_page(session_id, page_index, scale)
    }

    pub(crate) fn close(&self, session_id: &str) -> AppResult<()> {
        let result = self.engine.close(session_id);
        if result.is_ok() {
            self.managed_documents
                .lock()
                .map_err(|_| AppError::StateUnavailable)?
                .remove(session_id);
        }
        result
    }
}

fn apply_managed_identity(document: &mut PdfDocumentModel, managed: &ManagedPdfDocument) {
    document.document_id = managed.document_id.clone();
    document.path = managed.storage_path.clone();
    document.source_path = managed.source_path.clone();
    document.title = managed.display_name.clone();
}
