use std::sync::Arc;

use tauri::State;

use crate::{
    commands::dto::{PdfBookmarkDto, PdfReadingStateDto},
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn persist_pdf_document(state: State<'_, AppState>, session_id: String) -> AppResult<()> {
    let service = Arc::clone(&state.pdf_documents);
    let settings = state.settings.load()?;
    tauri::async_runtime::spawn_blocking(move || service.persist(&session_id, &settings).map(|_| ()))
        .await.map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn load_pdf_bookmarks(
    state: State<'_, AppState>,
    document_id: String,
    source_path: Option<String>,
) -> AppResult<Vec<PdfBookmarkDto>> {
    let storage = Arc::clone(&state.storage);
    tauri::async_runtime::spawn_blocking(move || {
        storage
            .load_pdf_bookmarks(&document_id, source_path.as_deref())
            .map(|items| items.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_pdf_bookmarks(
    state: State<'_, AppState>,
    document_id: String,
    source_path: Option<String>,
    bookmarks: Vec<PdfBookmarkDto>,
) -> AppResult<()> {
    let storage = Arc::clone(&state.storage);
    let bookmarks = bookmarks.into_iter().map(Into::into).collect::<Vec<_>>();
    tauri::async_runtime::spawn_blocking(move || {
        storage.save_pdf_bookmarks(&document_id, source_path.as_deref(), &bookmarks)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn load_pdf_reading_state(
    state: State<'_, AppState>,
    document_id: String,
    source_path: Option<String>,
) -> AppResult<Option<PdfReadingStateDto>> {
    let storage = Arc::clone(&state.storage);
    tauri::async_runtime::spawn_blocking(move || {
        storage
            .load_pdf_reading_state(&document_id, source_path.as_deref())
            .map(|value| value.map(Into::into))
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_pdf_reading_state(
    state: State<'_, AppState>,
    document_id: String,
    source_path: Option<String>,
    reading_state: PdfReadingStateDto,
) -> AppResult<()> {
    let storage = Arc::clone(&state.storage);
    tauri::async_runtime::spawn_blocking(move || {
        storage.save_pdf_reading_state(&document_id, source_path.as_deref(), &reading_state.into())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
