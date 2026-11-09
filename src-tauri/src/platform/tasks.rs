use crate::shared::error::{AppError, AppResult};

pub(crate) async fn run_blocking<T: Send + 'static>(
    operation: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    // Tauri是使用了Tokio异步运行时，所以这里需要使用spawn_blocking来将阻塞的操作放到一个单独的线程中执行
    // 这里的operation是一个闭包函数，它返回一个AppResult<T>类型的值，其中T是操作的返回值类型
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
