use std::{future::Future, pin::Pin, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    application::ports::LlmClient,
    domain::llm::{LlmChatRequest, LlmChatResponse, LlmConfig, LlmConnectionStatus},
    error::{AppError, AppResult},
};

pub(crate) struct OpenAiCompatibleLlmClient {
    client: reqwest::Client,
}

const MAX_STREAM_BUFFER_BYTES: usize = 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 2 * 1024 * 1024;
const MAX_OUTPUT_TOKENS: u32 = 8192;


impl OpenAiCompatibleLlmClient {
    pub(crate) fn new() -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|error| AppError::Llm(format!("无法创建 OpenAI 兼容客户端：{error}")))?;
        Ok(Self { client })
    }
}

impl LlmClient for OpenAiCompatibleLlmClient {
    fn id(&self) -> &'static str {
        "OpenAI-compatible"
    }

    fn chat_stream<'a>(
        &'a self,
        config: &'a LlmConfig,
        request: LlmChatRequest,
        on_delta: &'a mut (dyn FnMut(String) -> AppResult<()> + Send),
    ) -> Pin<Box<dyn Future<Output = AppResult<LlmChatResponse>> + Send + 'a>> {
        Box::pin(async move {
            let endpoint = endpoint(config, "/v1/chat/completions")?;
            let body = ChatCompletionRequest {
                model: &config.model,
                messages: request.messages.iter().map(|message| ChatMessage {
                    role: message.role.as_str(),
                    content: message.content.trim(),
                }).collect(),
                temperature: request.temperature,
                max_tokens: MAX_OUTPUT_TOKENS,
                stream: true,
            };
            let mut request = self.client.post(endpoint).timeout(Duration::from_secs(config.timeout_seconds)).json(&body);
            if !config.api_key.trim().is_empty() {
                request = request.bearer_auth(config.api_key.trim());
            }
            let mut response = request.send().await
                .map_err(|error| request_error(error, "请求大模型服务失败"))?;
            let status = response.status();
            if !status.is_success() {
                let payload = response.text().await
                    .map_err(|error| AppError::Llm(format!("无法读取大模型服务响应：{error}")))?;
                return Err(AppError::Llm(format!("大模型服务返回 HTTP {status}：{}", response_error(&payload))));
            }

            let mut pending = Vec::new();
            let mut output_bytes = 0_usize;
            let mut has_content = false;
            let mut model = config.model.clone();
            let mut usage = None;
            let mut finished = false;
            while !finished {
                let Some(chunk) = response.chunk().await
                    .map_err(|error| request_error(error, "读取大模型服务流式响应失败"))? else { break };
                pending.extend_from_slice(&chunk);
                if pending.len() > MAX_STREAM_BUFFER_BYTES {
                    return Err(AppError::Llm("大模型服务流式响应单帧过大，已中止请求".into()));
                }
                while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
                    let line = pending.drain(..=end).collect::<Vec<_>>();
                    let line = std::str::from_utf8(&line[..line.len() - 1])
                        .map_err(|error| AppError::Llm(format!("大模型服务流式响应不是 UTF-8：{error}")))?
                        .trim_end_matches('\r');
                    let Some(data) = line.strip_prefix("data:").map(str::trim_start) else { continue };
                    if data == "[DONE]" {
                        finished = true;
                        break;
                    }
                    let event: StreamChatCompletionResponse = serde_json::from_str(data)
                        .map_err(|error| AppError::Llm(format!("无法解析大模型服务流式响应：{error}")))?;
                    if !event.model.is_empty() {
                        model = event.model;
                    }
                    usage = event.usage.or(usage);
                    if let Some(delta) = event.choices.into_iter().find_map(|choice| choice.delta.content) {
                        output_bytes = output_bytes.saturating_add(delta.len());
                        if output_bytes > MAX_OUTPUT_BYTES {
                            return Err(AppError::Llm("大模型服务输出超过 2 MB，已中止请求".into()));
                        }
                        has_content |= !delta.is_empty();
                        on_delta(delta)?;
                    }
                }
            }
            if !has_content {
                return Err(AppError::Llm("大模型服务未返回可用回答".into()));
            }
            Ok(LlmChatResponse {
                content: String::new(),
                model,
                prompt_tokens: usage.as_ref().and_then(|usage: &Usage| usage.prompt_tokens),
                completion_tokens: usage.as_ref().and_then(|usage: &Usage| usage.completion_tokens),
                total_tokens: usage.and_then(|usage: Usage| usage.total_tokens),
            })
        })
    }

    fn test_connection<'a>(
        &'a self,
        config: &'a LlmConfig,
    ) -> Pin<Box<dyn Future<Output = AppResult<LlmConnectionStatus>> + Send + 'a>> {
        Box::pin(async move {
            let endpoint = endpoint(config, "/v1/models")?;
            let mut request = self.client.get(endpoint).timeout(Duration::from_secs(config.timeout_seconds.min(15)));
            if !config.api_key.trim().is_empty() {
                request = request.bearer_auth(config.api_key.trim());
            }
            let response = request.send().await
                .map_err(|error| request_error(error, "无法连接大模型服务"))?;
            let status = response.status();
            if status.is_success() {
                return Ok(LlmConnectionStatus { provider: self.id().into(), message: "OpenAI 兼容大模型服务连接正常".into() });
            }
            Err(AppError::Llm(format!("大模型服务连接测试返回 HTTP {status}")))
        })
    }
}

fn endpoint(config: &LlmConfig, suffix: &str) -> AppResult<String> {
    let base = config.base_url.trim_end_matches('/');
    let suffix = if base.ends_with("/v1") { suffix.trim_start_matches("/v1") } else { suffix };
    Ok(format!("{base}{suffix}"))
}

fn request_error(error: reqwest::Error, label: &str) -> AppError {
    if error.is_timeout() {
        AppError::Llm("大模型服务请求超时".into())
    } else {
        AppError::Llm(format!("{label}：{error}"))
    }
}

fn response_error(body: &str) -> String {
    const LIMIT: usize = 500;
    let trimmed = body.trim();
    let mut chars = trimmed.chars();
    let head = chars.by_ref().take(LIMIT).collect::<String>();
    if chars.next().is_none() { trimmed.into() } else { format!("{head}…") }
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct StreamChatCompletionResponse {
    #[serde(default)]
    model: String,
    #[serde(default)]
    choices: Vec<StreamChatChoice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct StreamChatChoice {
    #[serde(default)]
    delta: StreamChatResponseDelta,
}

#[derive(Default, Deserialize)]
struct StreamChatResponseDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Clone, Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::response_error;

    #[test]
    fn truncates_server_errors_before_returning_them_to_the_ui() {
        assert!(response_error(&"a".repeat(501)).ends_with('…'));
    }
}
