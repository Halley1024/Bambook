use crate::features::documents::PersistedDocumentKindDto;
use serde::{Deserialize, Serialize};

use crate::features::history::DocumentAvailability;
use crate::features::history::DocumentHistory;
use crate::features::history::DocumentHistoryEntry;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentHistoryDto {
    pub(crate) recent: Vec<DocumentHistoryEntryDto>,
    pub(crate) closed: Vec<DocumentHistoryEntryDto>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentHistoryEntryDto {
    #[serde(default)]
    document_id: Option<String>,
    path: String,
    title: String,
    kind: PersistedDocumentKindDto,
    last_closed_at: u64,
    #[serde(default)]
    availability: DocumentAvailabilityDto,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClosedDocumentInputDto {
    #[serde(default)]
    pub(crate) document_id: Option<String>,
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) kind: PersistedDocumentKindDto,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DocumentAvailabilityDto {
    Available,
    Missing,
    Inaccessible,
    NotFile,
    #[default]
    Unknown,
}

impl From<DocumentHistory> for DocumentHistoryDto {
    fn from(value: DocumentHistory) -> Self {
        Self {
            recent: value.recent.into_iter().map(Into::into).collect(),
            closed: value.closed.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DocumentHistoryDto> for DocumentHistory {
    fn from(value: DocumentHistoryDto) -> Self {
        Self {
            recent: value.recent.into_iter().map(Into::into).collect(),
            closed: value.closed.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DocumentHistoryEntry> for DocumentHistoryEntryDto {
    fn from(value: DocumentHistoryEntry) -> Self {
        Self {
            document_id: value.document_id,
            path: value.path,
            title: value.title,
            kind: value.kind.into(),
            last_closed_at: value.last_closed_at,
            availability: value.availability.into(),
        }
    }
}

impl From<DocumentHistoryEntryDto> for DocumentHistoryEntry {
    fn from(value: DocumentHistoryEntryDto) -> Self {
        Self {
            document_id: value.document_id,
            path: value.path,
            title: value.title,
            kind: value.kind.into(),
            last_closed_at: value.last_closed_at,
            availability: value.availability.into(),
        }
    }
}

impl From<DocumentAvailability> for DocumentAvailabilityDto {
    fn from(value: DocumentAvailability) -> Self {
        match value {
            DocumentAvailability::Available => Self::Available,
            DocumentAvailability::Missing => Self::Missing,
            DocumentAvailability::Inaccessible => Self::Inaccessible,
            DocumentAvailability::NotFile => Self::NotFile,
            DocumentAvailability::Unknown => Self::Unknown,
        }
    }
}

impl From<DocumentAvailabilityDto> for DocumentAvailability {
    fn from(value: DocumentAvailabilityDto) -> Self {
        match value {
            DocumentAvailabilityDto::Available => Self::Available,
            DocumentAvailabilityDto::Missing => Self::Missing,
            DocumentAvailabilityDto::Inaccessible => Self::Inaccessible,
            DocumentAvailabilityDto::NotFile => Self::NotFile,
            DocumentAvailabilityDto::Unknown => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentHistoryDto;
    use crate::features::workspace::WorkspaceStateDto;

    #[test]
    fn persistence_dtos_use_stable_camel_case_fields() {
        let history: DocumentHistoryDto = serde_json::from_value(serde_json::json!({
            "recent": [{
                "path": "C:\\docs\\sample.pdf",
                "title": "sample.pdf",
                "kind": "pdf",
                "lastClosedAt": 42
            }],
            "closed": []
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(history).unwrap()["recent"][0]["lastClosedAt"],
            42
        );

        let workspace: WorkspaceStateDto = serde_json::from_value(serde_json::json!({
            "openDocuments": [{ "path": "C:\\docs\\notes.md", "kind": "markdown" }],
            "activeDocumentPath": "C:\\docs\\notes.md",
            "folderPath": null
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(workspace).unwrap()["openDocuments"][0]["kind"],
            "markdown"
        );
    }
}
