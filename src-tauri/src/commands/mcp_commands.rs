use std::sync::Arc;

use tauri::State;

use crate::{commands::dto::McpToolDefinitionDto, error::AppResult, state::AppState};

/// Returns tools registered by an approved MCP client. The initial registry is intentionally empty.
#[tauri::command]
pub(crate) async fn list_mcp_tools(state: State<'_, AppState>) -> AppResult<Vec<McpToolDefinitionDto>> {
    let service = Arc::clone(&state.mcp);
    tauri::async_runtime::spawn_blocking(move || service.list_tools().map(|tools| tools.into_iter().map(Into::into).collect()))
        .await
        .map_err(|error| crate::error::AppError::BackgroundTask(error.to_string()))?
}
