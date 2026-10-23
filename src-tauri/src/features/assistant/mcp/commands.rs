use std::sync::Arc;

use tauri::State;

use crate::features::assistant::mcp::McpService;
use crate::features::assistant::mcp::McpToolDefinitionDto;
use crate::shared::error::AppResult;

/// Returns tools registered by an approved MCP client. The initial registry is intentionally empty.
#[tauri::command]
pub(crate) async fn list_mcp_tools(
    state: State<'_, Arc<McpService>>,
) -> AppResult<Vec<McpToolDefinitionDto>> {
    let service = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service
            .list_tools()
            .map(|tools| tools.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| crate::shared::error::AppError::BackgroundTask(error.to_string()))?
}
