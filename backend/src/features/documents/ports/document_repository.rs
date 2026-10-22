use crate::features::documents::DirectoryEntry;
use crate::features::documents::DocumentKind;
use crate::shared::error::AppResult;

pub(crate) trait DocumentRepository: Send + Sync {
    fn rename(&self, path: &str, new_path: &str) -> AppResult<()>;  // 重命名接口
    fn detect_kind(&self, path: &str) -> AppResult<DocumentKind>;   // 检测文件类型
    fn list_markdown_directory(&self, document_path: &str) -> AppResult<Vec<DirectoryEntry>>;   // 列出markdown文件夹
    fn list_supported_directory(&self, directory_path: &str) -> AppResult<Vec<DirectoryEntry>>; // 列出支持的文件夹
}
