use std::sync::Arc;

use crate::features::annotations::Annotation;
use crate::features::annotations::AnnotationExporter;
use crate::features::documents::ReaderDocument;
use crate::features::settings::AppSettings;
use crate::shared::error::AppResult;

pub(crate) struct ExportService {
    exporter: Arc<dyn AnnotationExporter>,
}

impl ExportService {
    pub(crate) fn new(exporter: Arc<dyn AnnotationExporter>) -> Self {
        Self { exporter }
    }

    pub(crate) fn export_annotations_markdown(
        &self,
        document: &ReaderDocument,
        annotations: &[Annotation],
        settings: &AppSettings,
    ) -> AppResult<String> {
        self.exporter
            .export_markdown(document, annotations, settings)
    }
}
