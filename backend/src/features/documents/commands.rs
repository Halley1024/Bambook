use std::sync::Arc;

use tauri::State;

use crate::features::documents::DirectoryEntryDto;
use crate::features::documents::DocumentKindDto;
use crate::features::documents::DocumentService;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn detect_document_kind(
    state: State<'_, Arc<DocumentService>>,
    path: String,
) -> AppResult<DocumentKindDto> {
    let service = Arc::clone(state.inner());    // 
    tauri::async_runtime::spawn_blocking(move || service.detect_kind(&path).map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn list_markdown_directory(
    state: State<'_, Arc<DocumentService>>,
    path: String,
) -> AppResult<Vec<DirectoryEntryDto>> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service
            .list_markdown_directory(&path)
            .map(|entries| entries.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn list_supported_directory(
    state: State<'_, Arc<DocumentService>>,
    path: String,
) -> AppResult<Vec<DirectoryEntryDto>> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service
            .list_supported_directory(&path)
            .map(|entries| entries.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn rename_markdown_document(
    state: State<'_, Arc<DocumentService>>,
    path: String,
    new_path: String,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    crate::platform::tasks::run_blocking(move || service.rename(&path, &new_path, true)).await
}

#[tauri::command]
pub(crate) async fn rename_pdf_document(
    state: State<'_, Arc<DocumentService>>,
    path: String,
    new_path: String,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    crate::platform::tasks::run_blocking(move || service.rename(&path, &new_path, false)).await
}
