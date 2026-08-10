use crate::domain::document::DocumentKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentHistoryEntry {
    pub(crate) document_id: Option<String>,
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) kind: DocumentKind,
    pub(crate) last_closed_at: u64,
    pub(crate) availability: DocumentAvailability,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum DocumentAvailability {
    Available,
    Missing,
    Inaccessible,
    NotFile,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DocumentHistory {
    pub(crate) recent: Vec<DocumentHistoryEntry>,
    pub(crate) closed: Vec<DocumentHistoryEntry>,
}
