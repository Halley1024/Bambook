use std::sync::Arc;

use crate::{
    application::ports::LlmClient,
    domain::llm::{
        LlmChatRequest, LlmChatResponse, LlmConfig, LlmConnectionStatus,
    },
    error::{AppError, AppResult},
};

pub(crate) struct LlmService {
    client: Arc<dyn LlmClient>,
}

impl LlmService {
    pub(crate) fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub(crate) async fn chat_stream(
        &self,
        config: &LlmConfig,
        request: LlmChatRequest,
        on_delta: &mut (dyn FnMut(String) -> AppResult<()> + Send),
    ) -> AppResult<LlmChatResponse> {
        validate_config(config)?;
        validate_request(&request)?;
        self.client.chat_stream(config, request, on_delta).await
    }

    pub(crate) async fn test_connection(
        &self,
        config: &LlmConfig,
    ) -> AppResult<LlmConnectionStatus> {
        validate_config(config)?;
        self.client.test_connection(config).await
    }
}

fn validate_config(config: &LlmConfig) -> AppResult<()> {
    let parsed = reqwest::Url::parse(config.base_url.trim())
        .map_err(|_| AppError::Llm("LLM 地址格式无效".into()))?;
    if parsed.host_str().is_none() {
        return Err(AppError::Llm("LLM 地址必须包含主机名".into()));
    }
    if parsed.scheme() == "http"
        && !matches!(parsed.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
    {
        return Err(AppError::Llm(
            "HTTP LLM 地址仅允许 localhost、127.0.0.1 或 ::1；远端服务必须使用 HTTPS".into(),
        ));
    }
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::Llm("LLM 地址必须使用 HTTP 或 HTTPS".into()));
    }
    if config.model.trim().is_empty() {
        return Err(AppError::Llm("请填写模型名称".into()));
    }
    if !(5..=600).contains(&config.timeout_seconds) {
        return Err(AppError::Llm("请求超时必须在 5 到 600 秒之间".into()));
    }
    Ok(())
}

fn validate_request(request: &LlmChatRequest) -> AppResult<()> {
    if request.messages.is_empty() || request.messages.len() > 32 {
        return Err(AppError::Llm("消息数量必须在 1 到 32 条之间".into()));
    }
    if !(0.0..=2.0).contains(&request.temperature) {
        return Err(AppError::Llm("temperature 必须在 0 到 2 之间".into()));
    }
    let total = request.messages.iter().try_fold(0_usize, |size, message| {
        let content = message.content.trim();
        if content.is_empty() {
            return Err(AppError::Llm("消息内容不能为空".into()));
        }
        Ok(size + content.len())
    })?;
    if total > 96_000 {
        return Err(AppError::Llm(
            "请求文本过长，请先通过 RAG 缩小上下文".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_config;
    use crate::domain::llm::LlmConfig;

    #[test]
    fn allows_loopback_http_and_remote_https() {
        let mut config = LlmConfig {
            base_url: "http://127.0.0.1:8080".into(),
            model: "local".into(),
            api_key: String::new(),
            timeout_seconds: 120,
        };
        assert!(validate_config(&config).is_ok());
        config.base_url = "http://192.168.1.10:8080".into();
        assert!(validate_config(&config).is_err());
        config.base_url = "https://api.example.com/v1".into();
        assert!(validate_config(&config).is_ok());
    }
}
