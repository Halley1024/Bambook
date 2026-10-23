use crate::app::{state::AppState, workflows};
use crate::features::history::DocumentHistoryDto;
use crate::features::markdown::MarkdownDocumentDto;
use crate::features::pdf::PdfDocumentDto;
use crate::platform::tasks::run_blocking;
use crate::shared::error::AppResult;
use tauri::State;

#[tauri::command]
pub(crate) async fn open_pdf_document(
    state: State<'_, AppState>,
    path: String,
    password: Option<String>,
) -> AppResult<PdfDocumentDto> {
    let state = state.inner().clone(); // 将里面的状态信息取出
    run_blocking(move || {
        workflows::pdf_lifecycle::open(&state, &path, password.as_deref()).map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn authenticate_pdf_document(
    state: State<'_, AppState>,
    session_id: String,
    password: String,
) -> AppResult<PdfDocumentDto> {
    let state = state.inner().clone();
    run_blocking(move || {
        workflows::pdf_lifecycle::authenticate(&state, &session_id, &password).map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn close_pdf_document(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || workflows::pdf_lifecycle::close(&state, &session_id)).await
}

#[tauri::command]
pub(crate) async fn persist_pdf_document(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || workflows::pdf_lifecycle::persist(&state, &session_id)).await
}

#[tauri::command]
pub(crate) async fn open_markdown_document(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<MarkdownDocumentDto> {
    let state = state.inner().clone();
    run_blocking(move || {
        workflows::document_lifecycle::open_markdown(&state, &path).map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn save_markdown_document(
    state: State<'_, AppState>,
    session_id: String,
    revision: u64,
) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || {
        workflows::document_lifecycle::save_markdown(&state, &session_id, revision)
    })
    .await
}

#[tauri::command]
pub(crate) async fn save_markdown_document_as(
    state: State<'_, AppState>,
    session_id: String,
    revision: u64,
    path: String,
) -> AppResult<MarkdownDocumentDto> {
    let state = state.inner().clone();
    run_blocking(move || {
        workflows::document_lifecycle::save_markdown_as(&state, &session_id, revision, &path)
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub(crate) async fn close_markdown_document(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || workflows::document_lifecycle::close_markdown(&state, &session_id)).await
}

#[tauri::command]
pub(crate) async fn load_document_history(
    state: State<'_, AppState>,
) -> AppResult<DocumentHistoryDto> {
    let state = state.inner().clone();
    run_blocking(move || workflows::history::load(&state).map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn export_document_package(
    state: State<'_, AppState>,
    document_id: String,
    destination: String,
) -> AppResult<String> {
    let state = state.inner().clone();
    run_blocking(move || workflows::history::export_package(&state, &document_id, &destination))
        .await
}

#[tauri::command]
pub(crate) async fn reconcile_document_storage(
    state: State<'_, AppState>,
    document_id: String,
) -> AppResult<()> {
    let state = state.inner().clone();
    run_blocking(move || workflows::document_lifecycle::reconcile_storage(&state, &document_id))
        .await
}
