use crate::{
    domain::{annotation::Annotation, document::ReaderDocument, settings::AppSettings},
    error::AppResult,
};

pub(crate) trait AnnotationExporter: Send + Sync {
    fn export_markdown(
        &self,
        document: &ReaderDocument,
        annotations: &[Annotation],
        settings: &AppSettings,
    ) -> AppResult<String>;
}
