use crate::features::documents::DocumentKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentHistoryEntry {
    pub(crate) document_id: Option<String>, // 已托管文档的标识；未纳入文档库时为空。
    pub(crate) path: String,                // 文档文件路径。
    pub(crate) title: String,               // 历史记录中显示的标题。
    pub(crate) kind: DocumentKind,          // 文档类型。
    pub(crate) last_closed_at: u64,         // 最近访问或关闭的时间，单位为 Unix 毫秒。
    pub(crate) availability: DocumentAvailability, // 当前文件路径的可用状态。
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
    pub(crate) recent: Vec<DocumentHistoryEntry>, // 按最近访问时间排序的文档记录。
    pub(crate) closed: Vec<DocumentHistoryEntry>, // 按关闭时间排序的文档记录。
}
