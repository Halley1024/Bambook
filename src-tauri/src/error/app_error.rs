use serde::ser::Serializer;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage {operation} failed for {path}: {source}", path = path.display())]
    StorageIo {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("path has no file name")]
    MissingFileName,
    #[error("path has no parent directory")]
    MissingParentDirectory,
    #[error("path is not a Markdown document")]
    InvalidMarkdownPath,
    #[error("export path must use the .{expected} extension")]
    InvalidExportExtension { expected: &'static str },
    #[error("generated {format} export failed validation")]
    InvalidExportOutput { format: &'static str },
    #[error("file type is not supported")]
    UnsupportedDocumentType,
    #[error("document id is invalid")]
    InvalidDocumentId,
    #[error("Bambook 文档库中已存在名为“{name}”的托管文档，不能重复保存")]
    DuplicateManagedDocumentName { name: String },
    #[error("application data directory is unavailable")]
    MissingAppDataDir,
    #[error("background task failed: {0}")]
    BackgroundTask(String),
    #[error("Markdown session was not found")]
    MarkdownSessionNotFound,
    #[error("Markdown revision conflict: expected {expected}, received {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    #[error("Markdown edit range is invalid")]
    InvalidMarkdownEdit,
    #[error("shared state is unavailable")]
    StateUnavailable,
    #[error("MuPDF error: {0}")]
    MuPdf(String),
    #[error("PDF session was not found")]
    PdfSessionNotFound,
    #[error("PDF page index is out of range")]
    PageOutOfRange,
    #[error("PDF password is required")]
    PdfPasswordRequired,
    #[error("PDF password is invalid")]
    PdfInvalidPassword,
    #[error("external link protocol is not allowed")]
    UnsafeExternalLink,
}

// Keep the current IPC error contract compatible while the frontend still
// displays rejected command values as strings. A structured CommandError DTO
// can replace this in a dedicated protocol migration.
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub(crate) type AppResult<T> = Result<T, AppError>;
