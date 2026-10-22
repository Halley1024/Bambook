use std::sync::Arc;

use tauri::{ipc::Response, State};

use crate::features::pdf::PdfDocumentService;
use crate::features::pdf::PdfPageStructureDto;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn get_pdf_page_structure(
    state: State<'_, Arc<PdfDocumentService>>,
    session_id: String,
    page_index: u32,
) -> AppResult<PdfPageStructureDto> {
    let service = Arc::clone(state.inner());
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
    state: State<'_, Arc<PdfDocumentService>>,
    session_id: String,
    page_index: u32,
    scale: f32,
) -> AppResult<Response> {
    let service = Arc::clone(state.inner());
    let rendered = tauri::async_runtime::spawn_blocking(move || {
        service.render_page(&session_id, page_index, scale)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))??;
    Ok(Response::new(rendered.png))
}
