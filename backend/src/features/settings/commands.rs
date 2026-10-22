use std::sync::Arc;

use tauri::State;

use crate::features::settings::AppSettingsDto;
use crate::features::settings::SettingsService;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn load_settings(
    state: State<'_, Arc<SettingsService>>,
) -> AppResult<AppSettingsDto> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.load().map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_settings(
    state: State<'_, Arc<SettingsService>>,
    settings: AppSettingsDto,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || service.save(&settings.into()))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
