use std::path::{Path, PathBuf};

use crate::features::documents::DocumentKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StorageLayout {
    pub(crate) root: PathBuf,                  // 应用数据根目录。
    pub(crate) storage: PathBuf,               // 托管文档的存储目录。
    pub(crate) themes: PathBuf,                // 主题文件目录。
    pub(crate) settings_file: PathBuf,         // 设置文件路径。
    pub(crate) recent_documents_file: PathBuf, // 最近文档记录文件路径。
    pub(crate) closed_documents_file: PathBuf, // 已关闭文档记录文件路径。
    pub(crate) workspace_state_file: PathBuf,  // 工作区状态文件路径。
    pub(crate) document_library_file: PathBuf, // 文档库索引文件路径。
}

impl StorageLayout {
    pub(crate) fn required_directories(&self) -> [&Path; 3] {
        [&self.root, &self.storage, &self.themes]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedPdfDocument {
    pub(crate) document_id: String,  // 托管文档标识。
    pub(crate) display_name: String, // 文档显示名称。
    pub(crate) source_path: String,  // 原始 PDF 文件路径。
    pub(crate) storage_path: String, // 本次打开时使用的 PDF 文件路径。
    pub(crate) data_path: String,    // 文档附属数据所在目录。
    pub(crate) storage_mode: String, // 存储方式标识。
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedMarkdownDocument {
    pub(crate) document_id: String,  // 托管文档标识。
    pub(crate) display_name: String, // 文档显示名称。
    pub(crate) source_path: String,  // 原始 Markdown 文件路径。
    pub(crate) storage_path: String, // 本次打开时使用的 Markdown 文件路径。
    pub(crate) data_path: String,    // 文档附属数据所在目录。
    pub(crate) storage_mode: String, // 存储方式标识。
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredDocumentEntry {
    pub(crate) id: String,           // 文档库中的稳定标识。
    pub(crate) title: String,        // 文档显示标题。
    pub(crate) kind: DocumentKind,   // 文档类型。
    pub(crate) path: String,         // 当前用于打开文档的文件路径。
    pub(crate) source_path: String,  // 导入时的原始文件路径。
    pub(crate) data_path: String,    // 文档附属数据所在目录。
    pub(crate) storage_mode: String, // 存储方式标识。
    pub(crate) created_at: u64,      // 文档入库时间，单位为 Unix 毫秒。
    pub(crate) updated_at: u64,      // 文档库记录更新时间，单位为 Unix 毫秒。
    pub(crate) last_opened_at: u64,  // 最近打开时间，单位为 Unix 毫秒。
}

pub(crate) struct RelinkInspection {
    pub(crate) recorded_name: String,
    pub(crate) selected_name: String,
    pub(crate) name_matches: bool,
}
