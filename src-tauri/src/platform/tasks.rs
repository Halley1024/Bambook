use crate::shared::error::{AppError, AppResult};

pub(crate) async fn run_blocking<T: Send + 'static>(
    operation: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
