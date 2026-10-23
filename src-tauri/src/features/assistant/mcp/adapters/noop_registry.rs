use crate::features::assistant::mcp::McpToolDefinition;
use crate::features::assistant::mcp::McpToolRegistry;
use crate::shared::error::AppResult;

/// Deliberately does not discover, start, or call MCP servers. It only preserves the boundary.
pub(crate) struct NoopMcpToolRegistry;

impl McpToolRegistry for NoopMcpToolRegistry {
    fn list_tools(&self) -> AppResult<Vec<McpToolDefinition>> {
        Ok(Vec::new())
    }
}
