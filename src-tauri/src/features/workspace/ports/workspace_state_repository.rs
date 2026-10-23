use crate::features::workspace::WorkspaceState;
use crate::shared::error::AppResult;

pub(crate) trait WorkspaceStateRepository: Send + Sync {
    fn load(&self) -> AppResult<WorkspaceState>;
    fn save(&self, state: &WorkspaceState) -> AppResult<()>;
}
