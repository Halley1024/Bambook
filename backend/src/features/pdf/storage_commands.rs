use std::sync::Arc;

use tauri::State;

use crate::features::pdf::PdfBookmarkDto;
use crate::features::pdf::PdfPersistenceService;
use crate::features::pdf::PdfReadingStateDto;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn load_pdf_bookmarks(
    state: State<'_, Arc<PdfPersistenceService>>,
    document_id: String,
    source_path: Option<String>,
) -> AppResult<Vec<PdfBookmarkDto>> {
    let storage = Arc::clone(state.inner());
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
    state: State<'_, Arc<PdfPersistenceService>>,
    document_id: String,
    source_path: Option<String>,
    bookmarks: Vec<PdfBookmarkDto>,
) -> AppResult<()> {
    let storage = Arc::clone(state.inner());
    let bookmarks = bookmarks.into_iter().map(Into::into).collect::<Vec<_>>();
    tauri::async_runtime::spawn_blocking(move || {
        storage.save_pdf_bookmarks(&document_id, source_path.as_deref(), &bookmarks)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn load_pdf_reading_state(
    state: State<'_, Arc<PdfPersistenceService>>,
    document_id: String,
    source_path: Option<String>,
) -> AppResult<Option<PdfReadingStateDto>> {
    let storage = Arc::clone(state.inner());
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
    state: State<'_, Arc<PdfPersistenceService>>,
    document_id: String,
    source_path: Option<String>,
    reading_state: PdfReadingStateDto,
) -> AppResult<()> {
    let storage = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        storage.save_pdf_reading_state(&document_id, source_path.as_deref(), &reading_state.into())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
