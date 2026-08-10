use std::sync::Arc;

use crate::{
    application::ports::AnnotationRepository,
    domain::{annotation::Annotation, settings::AppSettings},
    error::AppResult,
};

pub(crate) struct AnnotationService {
    repository: Arc<dyn AnnotationRepository>,
}

impl AnnotationService {
    pub(crate) fn new(repository: Arc<dyn AnnotationRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn load(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        settings: &AppSettings,
    ) -> AppResult<Vec<Annotation>> {
        self.repository.load(document_id, source_path, settings)
    }

    pub(crate) fn save(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        annotations: &[Annotation],
        settings: &AppSettings,
    ) -> AppResult<()> {
        self.repository.save(document_id, source_path, annotations, settings)
    }

    pub(crate) fn migrate(
        &self,
        from_document_id: &str,
        to_document_id: &str,
        settings: &AppSettings,
    ) -> AppResult<()> {
        self.repository
            .migrate(from_document_id, to_document_id, settings)
    }
}
