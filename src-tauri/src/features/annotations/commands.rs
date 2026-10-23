use std::sync::Arc;

use tauri::State;

use crate::features::annotations::AnnotationDto;
use crate::features::annotations::AnnotationService;
use crate::features::settings::AppSettingsDto;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn load_annotations(
    state: State<'_, Arc<AnnotationService>>,
    document_id: String,
    source_path: Option<String>,
    settings: AppSettingsDto,
) -> AppResult<Vec<AnnotationDto>> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service
            .load(&document_id, source_path.as_deref(), &settings.into())
            .map(|annotations| annotations.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_annotations(
    state: State<'_, Arc<AnnotationService>>,
    document_id: String,
    source_path: Option<String>,
    annotations: Vec<AnnotationDto>,
    settings: AppSettingsDto,
) -> AppResult<()> {
    let annotations = annotations.into_iter().map(Into::into).collect::<Vec<_>>();
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service.save(
            &document_id,
            source_path.as_deref(),
            &annotations,
            &settings.into(),
        )
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn migrate_pdf_annotations(
    state: State<'_, Arc<AnnotationService>>,
    from_document_id: String,
    to_document_id: String,
    settings: AppSettingsDto,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service.migrate(&from_document_id, &to_document_id, &settings.into())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
