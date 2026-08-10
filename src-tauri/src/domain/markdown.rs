#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownFileEntry {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) is_directory: bool,
    pub(crate) children: Vec<MarkdownFileEntry>,
}
