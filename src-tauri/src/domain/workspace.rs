use crate::domain::document::DocumentKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceDocument {
    pub(crate) path: String, // 工作区中打开的文档路径。
    pub(crate) kind: DocumentKind, // 文档类型。
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceState {
    pub(crate) open_documents: Vec<WorkspaceDocument>, // 工作区中打开的文档，按标签顺序排列。
    pub(crate) active_document_path: Option<String>, // 当前活动文档的路径；没有活动文档时为空。
    pub(crate) folder_path: Option<String>, // 当前打开的文件夹路径。
}
