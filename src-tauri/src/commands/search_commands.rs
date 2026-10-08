use std::sync::Arc;

use tauri::State;

use crate::{
    commands::dto::{CancelSearchRequestDto, SearchRequestDto, SearchResponseDto},
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn search_document(
    state: State<'_, AppState>,
    request: SearchRequestDto,
) -> AppResult<SearchResponseDto> {
    let service = Arc::clone(&state.search);
    tauri::async_runtime::spawn_blocking(move || service.search(request.into()).map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn cancel_document_search(
    state: State<'_, AppState>,
    request: CancelSearchRequestDto,
) -> AppResult<()> {
    let service = Arc::clone(&state.search);
    let (target, request_id) = request.into_parts();
    tauri::async_runtime::spawn_blocking(move || service.cancel(&target, request_id))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
