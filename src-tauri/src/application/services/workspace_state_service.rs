use std::sync::Arc;

use crate::{
    application::ports::WorkspaceStateRepository, domain::workspace::WorkspaceState,
    error::AppResult,
};

pub(crate) struct WorkspaceStateService {
    repository: Arc<dyn WorkspaceStateRepository>,
}

impl WorkspaceStateService {
    pub(crate) fn new(repository: Arc<dyn WorkspaceStateRepository>) -> Self {
        Self { repository }
    }

    pub(crate) fn load(&self) -> AppResult<WorkspaceState> {
        self.repository.load()
    }

    pub(crate) fn save(&self, state: &WorkspaceState) -> AppResult<()> {
        self.repository.save(state)
    }
}
