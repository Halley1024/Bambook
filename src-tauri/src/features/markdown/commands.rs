use std::sync::Arc;

use tauri::State;

use crate::features::markdown::MarkdownDocumentDto;
use crate::features::markdown::MarkdownDocumentService;
use crate::features::markdown::MarkdownEditDto;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn create_untitled_markdown_document(
    state: State<'_, Arc<MarkdownDocumentService>>,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.create_untitled().map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn get_markdown_structure(
    state: State<'_, Arc<MarkdownDocumentService>>,
    session_id: String,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.get(&session_id).map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn apply_markdown_edits(
    state: State<'_, Arc<MarkdownDocumentService>>,
    session_id: String,
    base_revision: u64,
    edits: Vec<MarkdownEditDto>,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(state.inner());
    let edits = edits.into_iter().map(Into::into).collect::<Vec<_>>();
    tauri::async_runtime::spawn_blocking(move || {
        service
            .apply_edits(&session_id, base_revision, &edits)
            .map(Into::into)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn export_markdown_html(
    state: State<'_, Arc<MarkdownDocumentService>>,
    session_id: String,
    revision: u64,
    path: String,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.export_html(&session_id, revision, &path))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn export_markdown_pdf(
    state: State<'_, Arc<MarkdownDocumentService>>,
    session_id: String,
    revision: u64,
    path: String,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.export_pdf(&session_id, revision, &path))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_untitled_markdown_recovery(
    state: State<'_, Arc<MarkdownDocumentService>>,
    content: String,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.save_untitled_recovery(&content))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn load_untitled_markdown_recovery(
    state: State<'_, Arc<MarkdownDocumentService>>,
) -> AppResult<Option<String>> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.load_untitled_recovery())
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn clear_untitled_markdown_recovery(
    state: State<'_, Arc<MarkdownDocumentService>>,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.clear_untitled_recovery())
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn create_markdown_document(
    state: State<'_, Arc<MarkdownDocumentService>>,
    path: String,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.create_file(&path))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
