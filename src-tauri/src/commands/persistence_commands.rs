use std::sync::Arc;

use tauri::State;
use serde::Serialize;

use crate::{
    commands::dto::{
        ClosedDocumentInputDto, DocumentHistoryDto, StoredDocumentEntryDto, WorkspaceStateDto,
    },
    domain::document_history::DocumentHistory,
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn load_document_history(
    state: State<'_, AppState>,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(&state.document_history);
    let storage = Arc::clone(&state.storage);
    run_blocking(move || {
        let library = storage.load_document_library()?;
        service.load_with_library(&library).map(Into::into)
    }).await
}

#[tauri::command]
pub(crate) async fn load_document_library(
    state: State<'_, AppState>,
) -> AppResult<Vec<StoredDocumentEntryDto>> {
    let service = Arc::clone(&state.storage);
    run_blocking(move || {
        service
            .load_document_library()
            .map(|documents| documents.into_iter().map(Into::into).collect())
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RelinkInspectionDto {
    recorded_name: String,
    selected_name: String,
    name_matches: bool,
}

#[tauri::command]
pub(crate) async fn inspect_document_relink(
    state: State<'_, AppState>, document_id: String, new_path: String,
) -> AppResult<RelinkInspectionDto> {
    let service = Arc::clone(&state.storage);
    run_blocking(move || {
        let entry = service.load_document_library()?.into_iter().find(|item| item.id == document_id)
            .ok_or(AppError::UnsupportedDocumentType)?;
        let selected_name = std::path::Path::new(&new_path).file_name().and_then(|value| value.to_str())
            .ok_or(AppError::MissingFileName)?.to_owned();
        Ok(RelinkInspectionDto { name_matches: entry.title.eq_ignore_ascii_case(&selected_name), recorded_name: entry.title, selected_name })
    }).await
}

#[tauri::command]
pub(crate) async fn relink_document(
    state: State<'_, AppState>, document_id: String, new_path: String,
) -> AppResult<StoredDocumentEntryDto> {
    let service = Arc::clone(&state.storage);
    run_blocking(move || service.relink_document(&document_id, &new_path).map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn reconcile_document_storage(
    state: State<'_, AppState>, document_id: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.storage);
    let settings = state.settings.load()?;
    run_blocking(move || service.reconcile_document_storage(&document_id, &settings)).await
}

#[tauri::command]
pub(crate) async fn export_document_package(
    state: State<'_, AppState>, document_id: String, destination: String,
) -> AppResult<String> {
    let storage = Arc::clone(&state.storage);
    let history = Arc::clone(&state.document_history);
    run_blocking(move || {
        let package_path = storage.export_document_package(&document_id, &destination)?;
        let package_document_id = if document_id.ends_with("-package") {
            document_id
        } else {
            format!("{document_id}-package")
        };
        let package = storage.load_document_library()?.into_iter()
            .find(|entry| entry.id == package_document_id)
            .ok_or(AppError::UnsupportedDocumentType)?;
        history.record_closed(
            Some(package.id),
            package.path,
            package.title,
            package.kind,
        )?;
        Ok(package_path)
    }).await
}

#[tauri::command]
pub(crate) async fn record_document_closed(
    state: State<'_, AppState>,
    document: ClosedDocumentInputDto,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(&state.document_history);
    run_blocking(move || {
        service
            .record_closed(document.document_id, document.path, document.title, document.kind.into())
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn record_document_accessed(
    state: State<'_, AppState>,
    document: ClosedDocumentInputDto,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(&state.document_history);
    run_blocking(move || {
        service
            .record_accessed(document.document_id, document.path, document.title, document.kind.into())
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn confirm_reopened_closed_document(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(&state.document_history);
    run_blocking(move || service.confirm_reopened(&path).map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn clear_document_history(
    state: State<'_, AppState>,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(&state.document_history);
    run_blocking(move || service.clear().map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn import_legacy_document_history(
    state: State<'_, AppState>,
    history: DocumentHistoryDto,
) -> AppResult<DocumentHistoryDto> {
    let service = Arc::clone(&state.document_history);
    let history: DocumentHistory = history.into();
    run_blocking(move || {
        service
            .import_legacy(history.recent, history.closed)
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn load_workspace_state(
    state: State<'_, AppState>,
) -> AppResult<WorkspaceStateDto> {
    let service = Arc::clone(&state.workspace_state);
    run_blocking(move || service.load().map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn save_workspace_state(
    state: State<'_, AppState>,
    workspace: WorkspaceStateDto,
) -> AppResult<()> {
    let service = Arc::clone(&state.workspace_state);
    run_blocking(move || service.save(&workspace.into())).await
}

async fn run_blocking<T: Send + 'static>(
    operation: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
