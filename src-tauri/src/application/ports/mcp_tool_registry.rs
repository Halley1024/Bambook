use crate::{domain::mcp::McpToolDefinition, error::AppResult};

/// Future MCP transports can implement this port after they complete JSON-RPC initialization.
pub(crate) trait McpToolRegistry: Send + Sync {
    fn list_tools(&self) -> AppResult<Vec<McpToolDefinition>>;
}
