use crate::{domain::document_history::DocumentHistoryEntry, error::AppResult};

pub(crate) trait DocumentHistoryRepository: Send + Sync {
    fn load_recent(&self) -> AppResult<Vec<DocumentHistoryEntry>>;
    fn save_recent(&self, entries: &[DocumentHistoryEntry]) -> AppResult<()>;
    fn load_closed(&self) -> AppResult<Vec<DocumentHistoryEntry>>;
    fn save_closed(&self, entries: &[DocumentHistoryEntry]) -> AppResult<()>;
}
