use crate::app::state::AppState;
use crate::features::history::DocumentHistory;
use crate::shared::error::{AppError, AppResult};

pub(crate) fn load(state: &AppState) -> AppResult<DocumentHistory> {
    let library = state.library.load_document_library()?;
    state.document_history.load_with_library(&library)
}
pub(crate) fn export_package(
    state: &AppState,
    document_id: &str,
    destination: &str,
) -> AppResult<String> {
    let package_path = state
        .library
        .export_document_package(document_id, destination)?;
    let package_id = if document_id.ends_with("-package") {
        document_id.to_owned()
    } else {
        format!("{document_id}-package")
    };
    let package = state
        .library
        .load_document_library()?
        .into_iter()
        .find(|entry| entry.id == package_id)
        .ok_or(AppError::UnsupportedDocumentType)?;
    state.document_history.record_closed(
        Some(package.id),
        package.path,
        package.title,
        package.kind,
    )?;
    Ok(package_path)
}
