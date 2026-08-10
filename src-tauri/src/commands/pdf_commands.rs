use std::sync::Arc;

use tauri::{ipc::Response, State};

use crate::{
    commands::dto::{PdfDocumentDto, PdfPageStructureDto},
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn open_pdf_document(
    state: State<'_, AppState>,
    path: String,
    password: Option<String>,
) -> AppResult<PdfDocumentDto> {
    let service = Arc::clone(&state.pdf_documents);
    let settings = state.settings.load()?;
    let search = Arc::clone(&state.search);
    let search_password = password.clone();
    let document =
        tauri::async_runtime::spawn_blocking(move || service.open(&path, password.as_deref(), &settings))
            .await
            .map_err(|error| AppError::BackgroundTask(error.to_string()))??;
    let session_id = document.session_id.clone();
    let search_path = document.path.clone();
    if let Err(error) = tauri::async_runtime::spawn_blocking(move || {
        search.register_pdf(&session_id, &search_path, search_password.as_deref())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
    {
        let _ = state.pdf_documents.close(&document.session_id);
        return Err(error);
    }
    Ok(document.into())
}

#[tauri::command]
pub(crate) async fn authenticate_pdf_document(
    state: State<'_, AppState>,
    session_id: String,
    password: String,
) -> AppResult<PdfDocumentDto> {
    let service = Arc::clone(&state.pdf_documents);
    let search = Arc::clone(&state.search);
    let search_session_id = session_id.clone();
    let search_password = password.clone();
    let document =
        tauri::async_runtime::spawn_blocking(move || service.authenticate(&session_id, &password))
            .await
            .map_err(|error| AppError::BackgroundTask(error.to_string()))??;
    tauri::async_runtime::spawn_blocking(move || {
        search.authenticate_pdf(&search_session_id, &search_password)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))??;
    Ok(document.into())
}

#[tauri::command]
pub(crate) async fn get_pdf_page_structure(
    state: State<'_, AppState>,
    session_id: String,
    page_index: u32,
) -> AppResult<PdfPageStructureDto> {
    let service = Arc::clone(&state.pdf_documents);
    tauri::async_runtime::spawn_blocking(move || {
        service
            .page_structure(&session_id, page_index)
            .map(Into::into)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn render_pdf_page(
    state: State<'_, AppState>,
    session_id: String,
    page_index: u32,
    scale: f32,
) -> AppResult<Response> {
    let service = Arc::clone(&state.pdf_documents);
    let rendered = tauri::async_runtime::spawn_blocking(move || {
        service.render_page(&session_id, page_index, scale)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))??;
    Ok(Response::new(rendered.png))
}

#[tauri::command]
pub(crate) async fn close_pdf_document(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.pdf_documents);
    let search = Arc::clone(&state.search);
    let search_session_id = session_id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let search_result = search.close_pdf(&search_session_id);
        let document_result = service.close(&session_id);
        document_result.and(search_result)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
