#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReaderDocument {
    pub(crate) id: String,    // 文档在应用中的标识。
    pub(crate) title: String, // 文档的显示标题。
    pub(crate) path: String,  // 文档文件路径。
    pub(crate) kind: String,  // 文档类型标识。
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OpenedDocument {
    pub(crate) path: String, // 已打开文件的路径。
    pub(crate) name: String, // 已打开文件的名称。
    pub(crate) text: String, // 从文件读取的文本内容。
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DocumentKind {
    Pdf,    // PDF类型
    Markdown,   // Markdown类型
    // 后续支持的类型
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DirectoryEntry {
    pub(crate) name: String,                  // 文件或目录名称。
    pub(crate) path: String,                  // 文件或目录的完整路径。
    pub(crate) is_directory: bool,            // 是否为目录。
    pub(crate) children: Vec<DirectoryEntry>, // 目录下的 Markdown 文件和子目录；文件节点为空。
}
