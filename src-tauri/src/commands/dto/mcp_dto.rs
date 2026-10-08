use serde::Serialize;
use serde_json::Value;

use crate::domain::mcp::McpToolDefinition;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct McpToolDefinitionDto {
    name: String,
    description: String,
    input_schema: Value,
}

impl From<McpToolDefinition> for McpToolDefinitionDto {
    fn from(value: McpToolDefinition) -> Self {
        Self { name: value.name, description: value.description, input_schema: value.input_schema }
    }
}
