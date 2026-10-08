use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::{
    commands::dto::AppSettingsDto,
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn load_settings(state: State<'_, AppState>) -> AppResult<AppSettingsDto> {
    let service = Arc::clone(&state.settings);
    tauri::async_runtime::spawn_blocking(move || service.load().map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettingsDto,
) -> AppResult<()> {
    let service = Arc::clone(&state.settings);
    tauri::async_runtime::spawn_blocking(move || service.save(&settings.into()))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn open_app_data_directory(app: AppHandle) -> AppResult<()> {
    let path = app
        .path()
        .local_data_dir()
        .map_err(|_| AppError::MissingAppDataDir)?
        .join("Bambook");
    tauri::async_runtime::spawn_blocking(move || {
        std::process::Command::new("explorer.exe").arg(path).spawn()?;
        Ok(())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
