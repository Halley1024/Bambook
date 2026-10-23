use std::sync::Arc;

use tauri::State;

use crate::features::library::LibraryService;
use crate::features::library::StoredDocumentEntryDto;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn load_document_library(
    state: State<'_, Arc<LibraryService>>,
) -> AppResult<Vec<StoredDocumentEntryDto>> {
    let service = Arc::clone(state.inner());
    run_blocking(move || {
        service
            .load_document_library()
            .map(|documents| documents.into_iter().map(Into::into).collect())
    })
    .await
}

#[tauri::command]
pub(crate) async fn relink_document(
    state: State<'_, Arc<LibraryService>>,
    document_id: String,
    new_path: String,
) -> AppResult<StoredDocumentEntryDto> {
    let service = Arc::clone(state.inner());
    run_blocking(move || {
        service
            .relink_document(&document_id, &new_path)
            .map(Into::into)
    })
    .await
}

use crate::platform::tasks::run_blocking;

#[tauri::command]
pub(crate) async fn inspect_document_relink(
    state: State<'_, Arc<LibraryService>>,
    document_id: String,
    new_path: String,
) -> AppResult<RelinkInspectionDto> {
    let service = Arc::clone(state.inner());
    run_blocking(move || {
        service
            .inspect_relink(&document_id, &new_path)
            .map(Into::into)
    })
    .await
}

use crate::features::library::RelinkInspectionDto;
