use crate::platform::tasks::run_blocking;
use crate::shared::error::{AppError, AppResult};
use tauri::{AppHandle, Manager};

#[tauri::command]
pub(crate) async fn reveal_in_file_manager(path: String) -> AppResult<()> {
    run_blocking(move || super::reveal(&path)).await
}

#[tauri::command]
pub(crate) async fn open_directory_in_file_manager(path: String) -> AppResult<()> {
    run_blocking(move || super::open_directory(&path)).await
}

#[tauri::command]
pub(crate) async fn open_external_url(url: String) -> AppResult<()> {
    run_blocking(move || super::open_external(&url)).await
}

#[tauri::command]
pub(crate) async fn open_app_data_directory(app: AppHandle) -> AppResult<()> {
    let path = app
        .path()
        .local_data_dir()
        .map_err(|_| AppError::MissingAppDataDir)?
        .join("Bambook");
    run_blocking(move || super::open_directory(&path.to_string_lossy())).await
}
