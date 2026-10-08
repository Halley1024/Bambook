use std::sync::Arc;

use tauri::State;

use crate::{
    commands::dto::{AnnotationDto, AppSettingsDto, ReaderDocumentDto},
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn export_annotations_markdown(
    state: State<'_, AppState>,
    document: ReaderDocumentDto,
    annotations: Vec<AnnotationDto>,
    settings: AppSettingsDto,
) -> AppResult<String> {
    let document = document.into();
    let annotations = annotations.into_iter().map(Into::into).collect::<Vec<_>>();
    let service = Arc::clone(&state.exports);
    tauri::async_runtime::spawn_blocking(move || {
        service.export_annotations_markdown(&document, &annotations, &settings.into())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
