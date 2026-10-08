#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownFileEntry {
    pub(crate) name: String, // 文件或目录名称。
    pub(crate) path: String, // 文件或目录的完整路径。
    pub(crate) is_directory: bool, // 是否为目录。
    pub(crate) children: Vec<MarkdownFileEntry>, // 目录下的 Markdown 文件和子目录；文件节点为空。
}
