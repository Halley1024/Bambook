use std::sync::Arc;

use tauri::State;

use crate::features::history::ClosedDocumentInputDto;
use crate::features::history::DocumentHistory;
use crate::features::history::DocumentHistoryDto;
use crate::features::history::DocumentHistoryService;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn record_document_closed(
    state: State<'_, Arc<DocumentHistoryService>>,
    document: ClosedDocumentInputDto,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(state.inner());
    run_blocking(move || {
        service
            .record_closed(
                document.document_id,
                document.path,
                document.title,
                document.kind.into(),
            )
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn record_document_accessed(
    state: State<'_, Arc<DocumentHistoryService>>,
    document: ClosedDocumentInputDto,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(state.inner());
    run_blocking(move || {
        service
            .record_accessed(
                document.document_id,
                document.path,
                document.title,
                document.kind.into(),
            )
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn confirm_reopened_closed_document(
    state: State<'_, Arc<DocumentHistoryService>>,
    path: String,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(state.inner());
    run_blocking(move || service.confirm_reopened(&path).map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn clear_document_history(
    state: State<'_, Arc<DocumentHistoryService>>,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(state.inner());
    run_blocking(move || service.clear().map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn import_legacy_document_history(
    state: State<'_, Arc<DocumentHistoryService>>,
    history: DocumentHistoryDto,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(state.inner());
    let history: DocumentHistory = history.into();
    run_blocking(move || {
        service
            .import_legacy(history.recent, history.closed)
            .map(Into::into)
    })
    .await
}

use crate::platform::tasks::run_blocking;
