use serde_json::Value;

/// This is the MCP `tools/list` shape. Execution is intentionally not enabled yet.
#[derive(Debug, Clone)]
pub(crate) struct McpToolDefinition {
    pub(crate) name: String, // MCP 工具名称。
    pub(crate) description: String, // 工具用途说明。
    pub(crate) input_schema: Value, // 工具输入参数的 JSON Schema。
}
