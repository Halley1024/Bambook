use std::sync::Arc;

use tauri::State;

use crate::features::annotations::AnnotationDto;
use crate::features::annotations::ExportService;
use crate::features::documents::ReaderDocumentDto;
use crate::features::settings::AppSettingsDto;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn export_annotations_markdown(
    state: State<'_, Arc<ExportService>>,
    document: ReaderDocumentDto,
    annotations: Vec<AnnotationDto>,
    settings: AppSettingsDto,
) -> AppResult<String> {
    let document = document.into();
    let annotations = annotations.into_iter().map(Into::into).collect::<Vec<_>>();
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service.export_annotations_markdown(&document, &annotations, &settings.into())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
