use serde::{Deserialize, Serialize};

use crate::domain::markdown_ast::{
    MarkdownDiagnostic, MarkdownDocumentModel, MarkdownEdit, MarkdownNode, MarkdownNodeKind,
    MarkdownOutlineNode, SourceRange,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MarkdownDocumentDto {
    document_id: String,
    session_id: String,
    revision: u64,
    path: String,
    title: String,
    source: String,
    nodes: Vec<MarkdownNodeDto>,
    outline: Vec<MarkdownOutlineNodeDto>,
    diagnostics: Vec<MarkdownDiagnosticDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceRangeDto {
    start: usize,
    end: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarkdownNodeDto {
    id: String,
    range: SourceRangeDto,
    #[serde(flatten)]
    kind: MarkdownNodeKindDto,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum MarkdownNodeKindDto {
    Paragraph {
        children: Vec<MarkdownNodeDto>,
    },
    Heading {
        level: u8,
        children: Vec<MarkdownNodeDto>,
    },
    BlockQuote {
        children: Vec<MarkdownNodeDto>,
    },
    CodeBlock {
        language: Option<String>,
        children: Vec<MarkdownNodeDto>,
    },
    List {
        start: Option<u64>,
        children: Vec<MarkdownNodeDto>,
    },
    ListItem {
        children: Vec<MarkdownNodeDto>,
    },
    FootnoteDefinition {
        label: String,
        children: Vec<MarkdownNodeDto>,
    },
    DefinitionList {
        children: Vec<MarkdownNodeDto>,
    },
    DefinitionTitle {
        children: Vec<MarkdownNodeDto>,
    },
    Definition {
        children: Vec<MarkdownNodeDto>,
    },
    Table {
        alignments: Vec<String>,
        children: Vec<MarkdownNodeDto>,
    },
    TableHead {
        children: Vec<MarkdownNodeDto>,
    },
    TableRow {
        children: Vec<MarkdownNodeDto>,
    },
    TableCell {
        children: Vec<MarkdownNodeDto>,
    },
    Emphasis {
        children: Vec<MarkdownNodeDto>,
    },
    Strong {
        children: Vec<MarkdownNodeDto>,
    },
    Strikethrough {
        children: Vec<MarkdownNodeDto>,
    },
    Superscript {
        children: Vec<MarkdownNodeDto>,
    },
    Subscript {
        children: Vec<MarkdownNodeDto>,
    },
    Link {
        url: String,
        title: String,
        children: Vec<MarkdownNodeDto>,
    },
    Image {
        url: String,
        title: String,
        children: Vec<MarkdownNodeDto>,
    },
    Metadata {
        children: Vec<MarkdownNodeDto>,
    },
    Text {
        text: String,
    },
    InlineCode {
        text: String,
    },
    Html {
        text: String,
    },
    Math {
        text: String,
        display: bool,
    },
    FootnoteReference {
        label: String,
    },
    SoftBreak,
    HardBreak,
    Rule,
    TaskListMarker {
        checked: bool,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarkdownOutlineNodeDto {
    id: String,
    node_id: String,
    level: u8,
    title: String,
    children: Vec<MarkdownOutlineNodeDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarkdownDiagnosticDto {
    severity: String,
    message: String,
    range: Option<SourceRangeDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MarkdownEditDto {
    start_utf16: usize,
    end_utf16: usize,
    text: String,
}

impl From<MarkdownDocumentModel> for MarkdownDocumentDto {
    fn from(value: MarkdownDocumentModel) -> Self {
        let mut nodes = value.nodes;
        normalize_node_ranges(&mut nodes, &value.source);
        let diagnostics = value
            .diagnostics
            .into_iter()
            .map(|mut diagnostic| {
                if let Some(range) = diagnostic.range.as_mut() {
                    normalize_range(range, &value.source);
                }
                diagnostic.into()
            })
            .collect();
        Self {
            document_id: value.document_id,
            session_id: value.session_id,
            revision: value.revision,
            path: value.path,
            title: value.title,
            source: value.source,
            nodes: nodes.into_iter().map(Into::into).collect(),
            outline: value.outline.into_iter().map(Into::into).collect(),
            diagnostics,
        }
    }
}

fn normalize_node_ranges(nodes: &mut [MarkdownNode], source: &str) {
    for node in nodes {
        normalize_range(&mut node.range, source);
        match &mut node.kind {
            MarkdownNodeKind::Paragraph { children }
            | MarkdownNodeKind::Heading { children, .. }
            | MarkdownNodeKind::BlockQuote { children }
            | MarkdownNodeKind::CodeBlock { children, .. }
            | MarkdownNodeKind::List { children, .. }
            | MarkdownNodeKind::ListItem { children }
            | MarkdownNodeKind::FootnoteDefinition { children, .. }
            | MarkdownNodeKind::DefinitionList { children }
            | MarkdownNodeKind::DefinitionTitle { children }
            | MarkdownNodeKind::Definition { children }
            | MarkdownNodeKind::Table { children, .. }
            | MarkdownNodeKind::TableHead { children }
            | MarkdownNodeKind::TableRow { children }
            | MarkdownNodeKind::TableCell { children }
            | MarkdownNodeKind::Emphasis { children }
            | MarkdownNodeKind::Strong { children }
            | MarkdownNodeKind::Strikethrough { children }
            | MarkdownNodeKind::Superscript { children }
            | MarkdownNodeKind::Subscript { children }
            | MarkdownNodeKind::Link { children, .. }
            | MarkdownNodeKind::Image { children, .. }
            | MarkdownNodeKind::Metadata { children } => normalize_node_ranges(children, source),
            _ => {}
        }
    }
}

fn normalize_range(range: &mut SourceRange, source: &str) {
    range.start = byte_offset_to_utf16(source, range.start);
    range.end = byte_offset_to_utf16(source, range.end);
}

fn byte_offset_to_utf16(source: &str, byte_offset: usize) -> usize {
    source[..byte_offset].encode_utf16().count()
}

impl From<SourceRange> for SourceRangeDto {
    fn from(value: SourceRange) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<MarkdownNode> for MarkdownNodeDto {
    fn from(value: MarkdownNode) -> Self {
        Self {
            id: value.id,
            range: value.range.into(),
            kind: value.kind.into(),
        }
    }
}

fn nodes(value: Vec<MarkdownNode>) -> Vec<MarkdownNodeDto> {
    value.into_iter().map(Into::into).collect()
}

impl From<MarkdownNodeKind> for MarkdownNodeKindDto {
    fn from(value: MarkdownNodeKind) -> Self {
        match value {
            MarkdownNodeKind::Paragraph { children } => Self::Paragraph {
                children: nodes(children),
            },
            MarkdownNodeKind::Heading { level, children } => Self::Heading {
                level,
                children: nodes(children),
            },
            MarkdownNodeKind::BlockQuote { children } => Self::BlockQuote {
                children: nodes(children),
            },
            MarkdownNodeKind::CodeBlock { language, children } => Self::CodeBlock {
                language,
                children: nodes(children),
            },
            MarkdownNodeKind::List { start, children } => Self::List {
                start,
                children: nodes(children),
            },
            MarkdownNodeKind::ListItem { children } => Self::ListItem {
                children: nodes(children),
            },
            MarkdownNodeKind::FootnoteDefinition { label, children } => Self::FootnoteDefinition {
                label,
                children: nodes(children),
            },
            MarkdownNodeKind::DefinitionList { children } => Self::DefinitionList {
                children: nodes(children),
            },
            MarkdownNodeKind::DefinitionTitle { children } => Self::DefinitionTitle {
                children: nodes(children),
            },
            MarkdownNodeKind::Definition { children } => Self::Definition {
                children: nodes(children),
            },
            MarkdownNodeKind::Table {
                alignments,
                children,
            } => Self::Table {
                alignments,
                children: nodes(children),
            },
            MarkdownNodeKind::TableHead { children } => Self::TableHead {
                children: nodes(children),
            },
            MarkdownNodeKind::TableRow { children } => Self::TableRow {
                children: nodes(children),
            },
            MarkdownNodeKind::TableCell { children } => Self::TableCell {
                children: nodes(children),
            },
            MarkdownNodeKind::Emphasis { children } => Self::Emphasis {
                children: nodes(children),
            },
            MarkdownNodeKind::Strong { children } => Self::Strong {
                children: nodes(children),
            },
            MarkdownNodeKind::Strikethrough { children } => Self::Strikethrough {
                children: nodes(children),
            },
            MarkdownNodeKind::Superscript { children } => Self::Superscript {
                children: nodes(children),
            },
            MarkdownNodeKind::Subscript { children } => Self::Subscript {
                children: nodes(children),
            },
            MarkdownNodeKind::Link {
                url,
                title,
                children,
            } => Self::Link {
                url,
                title,
                children: nodes(children),
            },
            MarkdownNodeKind::Image {
                url,
                title,
                children,
            } => Self::Image {
                url,
                title,
                children: nodes(children),
            },
            MarkdownNodeKind::Metadata { children } => Self::Metadata {
                children: nodes(children),
            },
            MarkdownNodeKind::Text { text } => Self::Text { text },
            MarkdownNodeKind::InlineCode { text } => Self::InlineCode { text },
            MarkdownNodeKind::Html { text } => Self::Html { text },
            MarkdownNodeKind::Math { text, display } => Self::Math { text, display },
            MarkdownNodeKind::FootnoteReference { label } => Self::FootnoteReference { label },
            MarkdownNodeKind::SoftBreak => Self::SoftBreak,
            MarkdownNodeKind::HardBreak => Self::HardBreak,
            MarkdownNodeKind::Rule => Self::Rule,
            MarkdownNodeKind::TaskListMarker { checked } => Self::TaskListMarker { checked },
        }
    }
}

impl From<MarkdownOutlineNode> for MarkdownOutlineNodeDto {
    fn from(value: MarkdownOutlineNode) -> Self {
        Self {
            id: value.id,
            node_id: value.node_id,
            level: value.level,
            title: value.title,
            children: value.children.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<MarkdownDiagnostic> for MarkdownDiagnosticDto {
    fn from(value: MarkdownDiagnostic) -> Self {
        Self {
            severity: value.severity,
            message: value.message,
            range: value.range.map(Into::into),
        }
    }
}

impl From<MarkdownEditDto> for MarkdownEdit {
    fn from(value: MarkdownEditDto) -> Self {
        Self {
            start_utf16: value.start_utf16,
            end_utf16: value.end_utf16,
            text: value.text,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::markdown_ast::{
        MarkdownDocumentModel, MarkdownNode, MarkdownNodeKind, SourceRange,
    };

    use super::MarkdownDocumentDto;

    #[test]
    fn exposes_source_ranges_as_javascript_utf16_offsets() {
        let source = "😀\n# 标题".to_string();
        let dto = MarkdownDocumentDto::from(MarkdownDocumentModel {
            document_id: "md-test".into(),
            session_id: "markdown-1".into(),
            revision: 1,
            path: "notes.md".into(),
            title: "notes.md".into(),
            nodes: vec![MarkdownNode {
                id: "heading".into(),
                range: SourceRange {
                    start: 5,
                    end: source.len(),
                },
                kind: MarkdownNodeKind::Heading {
                    level: 1,
                    children: Vec::new(),
                },
            }],
            outline: Vec::new(),
            diagnostics: Vec::new(),
            source,
        });
        let json = serde_json::to_value(dto).unwrap();
        assert_eq!(json["nodes"][0]["range"]["start"], 3);
    }
}
