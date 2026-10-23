use crate::features::annotations::Annotation;
use crate::features::settings::AppSettings;
use crate::shared::error::AppResult;

pub(crate) trait AnnotationRepository: Send + Sync {
    fn load(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        settings: &AppSettings,
    ) -> AppResult<Vec<Annotation>>;
    fn save(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        annotations: &[Annotation],
        settings: &AppSettings,
    ) -> AppResult<()>;
    fn migrate(
        &self,
        from_document_id: &str,
        to_document_id: &str,
        settings: &AppSettings,
    ) -> AppResult<()>;
}
