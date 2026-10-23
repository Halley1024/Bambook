use std::sync::Arc;

use tauri::State;

use crate::features::workspace::WorkspaceStateDto;
use crate::features::workspace::WorkspaceStateService;
use crate::shared::error::AppResult;

#[tauri::command]
pub(crate) async fn load_workspace_state(
    state: State<'_, Arc<WorkspaceStateService>>,
) -> AppResult<WorkspaceStateDto> {
    let service = Arc::clone(state.inner());
    run_blocking(move || service.load().map(Into::into)).await
}

#[tauri::command]
pub(crate) async fn save_workspace_state(
    state: State<'_, Arc<WorkspaceStateService>>,
    workspace: WorkspaceStateDto,
) -> AppResult<()> {
    let service = Arc::clone(state.inner());
    run_blocking(move || service.save(&workspace.into())).await
}

use crate::platform::tasks::run_blocking;
