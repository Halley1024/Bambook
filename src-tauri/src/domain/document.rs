#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReaderDocument {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) path: String,
    pub(crate) kind: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OpenedDocument {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DocumentKind {
    Pdf,
    Markdown,
}
