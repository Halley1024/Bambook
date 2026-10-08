use serde::{Deserialize, Serialize};

use crate::domain::{
    document::{DocumentKind, ReaderDocument},
    markdown::MarkdownFileEntry,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentKindDto {
    kind: &'static str,
}

impl From<DocumentKind> for DocumentKindDto {
    fn from(value: DocumentKind) -> Self {
        Self {
            kind: match value {
                DocumentKind::Pdf => "pdf",
                DocumentKind::Markdown => "markdown",
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MarkdownFileEntryDto {
    name: String,
    path: String,
    is_directory: bool,
    children: Vec<MarkdownFileEntryDto>,
}

impl From<MarkdownFileEntry> for MarkdownFileEntryDto {
    fn from(value: MarkdownFileEntry) -> Self {
        Self {
            name: value.name,
            path: value.path,
            is_directory: value.is_directory,
            children: value.children.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReaderDocumentDto {
    id: String,
    title: String,
    path: String,
    kind: String,
}

impl From<ReaderDocumentDto> for ReaderDocument {
    fn from(value: ReaderDocumentDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            path: value.path,
            kind: value.kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DocumentKindDto, ReaderDocumentDto};

    #[test]
    fn ignores_document_payload_fields_not_needed_by_export() {
        let dto: ReaderDocumentDto = serde_json::from_value(serde_json::json!({
            "id": "doc-id",
            "title": "Sample",
            "path": "C:\\docs\\sample.pdf",
            "kind": "pdf",
            "bytes": [1, 2, 3]
        }))
        .unwrap();

        let document = crate::domain::document::ReaderDocument::from(dto);
        assert_eq!(document.title, "Sample");
        assert_eq!(document.kind, "pdf");
    }

    #[test]
    fn serializes_detected_document_kind_as_a_tagged_object() {
        let dto = DocumentKindDto::from(crate::domain::document::DocumentKind::Pdf);
        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            serde_json::json!({ "kind": "pdf" })
        );
    }
}
