use std::sync::Arc;

use crate::features::assistant::mcp::McpToolDefinition;
use crate::features::assistant::mcp::McpToolRegistry;
use crate::shared::error::AppResult;

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
