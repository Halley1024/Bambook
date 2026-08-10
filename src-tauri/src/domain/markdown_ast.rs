#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkdownDocumentModel {
    pub(crate) document_id: String,
    pub(crate) session_id: String,
    pub(crate) revision: u64,
    pub(crate) path: String,
    pub(crate) title: String,
    pub(crate) source: String,
    pub(crate) nodes: Vec<MarkdownNode>,
    pub(crate) outline: Vec<MarkdownOutlineNode>,
    pub(crate) diagnostics: Vec<MarkdownDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParsedMarkdown {
    pub(crate) nodes: Vec<MarkdownNode>,
    pub(crate) outline: Vec<MarkdownOutlineNode>,
    pub(crate) diagnostics: Vec<MarkdownDiagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkdownNode {
    pub(crate) id: String,
    pub(crate) range: SourceRange,
    pub(crate) kind: MarkdownNodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MarkdownNodeKind {
    Paragraph {
        children: Vec<MarkdownNode>,
    },
    Heading {
        level: u8,
        children: Vec<MarkdownNode>,
    },
    BlockQuote {
        children: Vec<MarkdownNode>,
    },
    CodeBlock {
        language: Option<String>,
        children: Vec<MarkdownNode>,
    },
    List {
        start: Option<u64>,
        children: Vec<MarkdownNode>,
    },
    ListItem {
        children: Vec<MarkdownNode>,
    },
    FootnoteDefinition {
        label: String,
        children: Vec<MarkdownNode>,
    },
    DefinitionList {
        children: Vec<MarkdownNode>,
    },
    DefinitionTitle {
        children: Vec<MarkdownNode>,
    },
    Definition {
        children: Vec<MarkdownNode>,
    },
    Table {
        alignments: Vec<String>,
        children: Vec<MarkdownNode>,
    },
    TableHead {
        children: Vec<MarkdownNode>,
    },
    TableRow {
        children: Vec<MarkdownNode>,
    },
    TableCell {
        children: Vec<MarkdownNode>,
    },
    Emphasis {
        children: Vec<MarkdownNode>,
    },
    Strong {
        children: Vec<MarkdownNode>,
    },
    Strikethrough {
        children: Vec<MarkdownNode>,
    },
    Superscript {
        children: Vec<MarkdownNode>,
    },
    Subscript {
        children: Vec<MarkdownNode>,
    },
    Link {
        url: String,
        title: String,
        children: Vec<MarkdownNode>,
    },
    Image {
        url: String,
        title: String,
        children: Vec<MarkdownNode>,
    },
    Metadata {
        children: Vec<MarkdownNode>,
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MarkdownOutlineNode {
    pub(crate) id: String,
    pub(crate) node_id: String,
    pub(crate) level: u8,
    pub(crate) title: String,
    pub(crate) children: Vec<MarkdownOutlineNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownDiagnostic {
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) range: Option<SourceRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownEdit {
    pub(crate) start_utf16: usize,
    pub(crate) end_utf16: usize,
    pub(crate) text: String,
}
