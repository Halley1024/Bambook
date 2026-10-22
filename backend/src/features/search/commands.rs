use std::sync::Arc;

use tauri::State;

use crate::features::search::CancelSearchRequestDto;
use crate::features::search::DocumentSearchService;
use crate::features::search::SearchRequestDto;
use crate::features::search::SearchResponseDto;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn search_document(
    state: State<'_, Arc<DocumentSearchService>>,
    request: SearchRequestDto,
) -> AppResult<SearchResponseDto> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.search(request.into()).map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn cancel_document_search(
    state: State<'_, Arc<DocumentSearchService>>,
    request: CancelSearchRequestDto,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    let (target, request_id) = request.into_parts();
    tauri::async_runtime::spawn_blocking(move || service.cancel(&target, request_id))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
