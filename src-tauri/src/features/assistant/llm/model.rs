#[derive(Debug, Clone)]
pub(crate) struct LlmConfig {
    pub(crate) base_url: String,     // 兼容 OpenAI 接口的服务根地址。
    pub(crate) model: String,        // 请求使用的模型名称。
    pub(crate) api_key: String,      // 访问服务所需的 API 密钥。
    pub(crate) timeout_seconds: u64, // 请求超时时长，单位为秒。
}

#[derive(Debug, Clone)]
pub(crate) struct LlmMessage {
    pub(crate) role: LlmMessageRole, // 消息在对话中的角色。
    pub(crate) content: String,      // 消息正文。
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LlmMessageRole {
    System,
    User,
    Assistant,
}

impl LlmMessageRole {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LlmChatRequest {
    pub(crate) messages: Vec<LlmMessage>, // 按对话顺序发送给模型的消息。
    pub(crate) temperature: f32,          // 生成时的温度参数。
}

#[derive(Debug, Clone)]
pub(crate) struct LlmChatResponse {
    pub(crate) content: String,                // 模型生成的回复文本。
    pub(crate) model: String,                  // 实际响应的模型名称。
    pub(crate) prompt_tokens: Option<u64>,     // 输入消息消耗的 token 数；服务未提供时为空。
    pub(crate) completion_tokens: Option<u64>, // 回复消耗的 token 数；服务未提供时为空。
    pub(crate) total_tokens: Option<u64>,      // 本次请求消耗的 token 总数；服务未提供时为空。
}

#[derive(Debug, Clone)]
pub(crate) struct LlmConnectionStatus {
    pub(crate) provider: String, // 被测试的模型服务提供方。
    pub(crate) message: String,  // 连接测试的结果说明。
}
