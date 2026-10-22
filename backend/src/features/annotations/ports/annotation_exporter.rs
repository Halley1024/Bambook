use crate::features::annotations::Annotation;
use crate::features::documents::ReaderDocument;
use crate::features::settings::AppSettings;
use crate::shared::error::AppResult;

pub(crate) trait AnnotationExporter: Send + Sync {
    fn export_markdown(
        &self,
        document: &ReaderDocument,
        annotations: &[Annotation],
        settings: &AppSettings,
    ) -> AppResult<String>;
}
