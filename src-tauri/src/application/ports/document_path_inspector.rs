use crate::domain::document_history::DocumentAvailability;

pub(crate) trait DocumentPathInspector: Send + Sync {
    fn inspect(&self, path: &str) -> DocumentAvailability;
}
