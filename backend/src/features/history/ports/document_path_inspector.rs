use crate::features::history::DocumentAvailability;

pub(crate) trait DocumentPathInspector: Send + Sync {
    fn inspect(&self, path: &str) -> DocumentAvailability;
}
