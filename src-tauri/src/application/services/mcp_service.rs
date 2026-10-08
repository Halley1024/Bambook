use std::sync::Arc;

use crate::{application::ports::McpToolRegistry, domain::mcp::McpToolDefinition, error::AppResult};

pub(crate) struct McpService {
    registry: Arc<dyn McpToolRegistry>,
}

impl McpService {
    pub(crate) fn new(registry: Arc<dyn McpToolRegistry>) -> Self {
        Self { registry }
    }

    pub(crate) fn list_tools(&self) -> AppResult<Vec<McpToolDefinition>> {
        self.registry.list_tools()
    }
}
