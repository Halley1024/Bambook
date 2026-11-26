use std::{future::Future, pin::Pin, time::Duration};

use serde::{Deserialize, Serialize};

use crate::features::assistant::llm::LlmChatRequest;
use crate::features::assistant::llm::LlmChatResponse;
use crate::features::assistant::llm::LlmClient;
use crate::features::assistant::llm::LlmConfig;
use crate::features::assistant::llm::LlmConnectionStatus;
use crate::features::assistant::llm::LlmStreamDelta;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;

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

// 实现兼容openAi接口的大模型客户端
impl LlmClient for OpenAiCompatibleLlmClient {
    fn id(&self) -> &'static str {
        "OpenAI-compatible"
    }

    fn chat_stream<'a>(
        &'a self,
        config: &'a LlmConfig,
        request: LlmChatRequest,
        on_delta: &'a mut (dyn FnMut(LlmStreamDelta) -> AppResult<()> + Send),
    ) -> Pin<Box<dyn Future<Output = AppResult<LlmChatResponse>> + Send + 'a>> {
        Box::pin(async move {
            let endpoint = endpoint(config, "/v1/chat/completions")?;
            let body = ChatCompletionRequest {
                model: &config.model,
                messages: request
                    .messages
                    .iter()
                    .map(|message| ChatMessage {
                        role: message.role.as_str(),
                        content: message.content.trim(),
                    })
                    .collect(),
                temperature: request.temperature,
                max_tokens: MAX_OUTPUT_TOKENS,
                enable_thinking: false,
                stream: true,
            };
            let mut request = self
                .client
                .post(endpoint)
                .timeout(Duration::from_secs(config.timeout_seconds))
                .json(&body);
            if !config.api_key.trim().is_empty() {
                request = request.bearer_auth(config.api_key.trim());
            }
            let mut response = request
                .send()
                .await
                .map_err(|error| request_error(error, "请求大模型服务失败"))?;
            let status = response.status();
            if !status.is_success() {
                let payload = response
                    .text()
                    .await
                    .map_err(|error| AppError::Llm(format!("无法读取大模型服务响应：{error}")))?;
                return Err(AppError::Llm(format!(
                    "大模型服务返回 HTTP {status}：{}",
                    response_error(&payload)
                )));
            }

            let mut pending = Vec::new();
            let mut output_bytes = 0_usize;
            let mut content = String::new();
            let mut has_reasoning = false;
            let mut model = config.model.clone();
            let mut usage = None;
            let mut finished = false;
            while !finished {
                let Some(chunk) = response
                    .chunk()
                    .await
                    .map_err(|error| request_error(error, "读取大模型服务流式响应失败"))?
                else {
                    break;
                };
                pending.extend_from_slice(&chunk);
                if pending.len() > MAX_STREAM_BUFFER_BYTES {
                    return Err(AppError::Llm(
                        "大模型服务流式响应单帧过大，已中止请求".into(),
                    ));
                }
                while let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
                    let line = pending.drain(..=end).collect::<Vec<_>>();
                    let line = std::str::from_utf8(&line[..line.len() - 1])
                        .map_err(|error| {
                            AppError::Llm(format!("大模型服务流式响应不是 UTF-8：{error}"))
                        })?
                        .trim_end_matches('\r');
                    let Some(data) = line.strip_prefix("data:").map(str::trim_start) else {
                        continue;
                    };
                    if data == "[DONE]" {
                        finished = true;
                        break;
                    }
                    let event: StreamChatCompletionResponse =
                        serde_json::from_str(data).map_err(|error| {
                            AppError::Llm(format!("无法解析大模型服务流式响应：{error}"))
                        })?;
                    if !event.model.is_empty() {
                        model = event.model;
                    }
                    usage = event.usage.or(usage);
                    // One chunk may contain reasoning and answer text together.
                    if let Some(choice) = event.choices.into_iter().next() {
                        for delta in choice.delta.into_events() {
                            let text = match &delta {
                                LlmStreamDelta::Content(text) | LlmStreamDelta::Reasoning(text) => {
                                    text
                                }
                            };
                            output_bytes = output_bytes.saturating_add(text.len());
                            if output_bytes > MAX_OUTPUT_BYTES {
                                return Err(AppError::Llm(
                                    "大模型服务输出超过 2 MB，已中止请求".into(),
                                ));
                            }
                            match &delta {
                                LlmStreamDelta::Content(text) => content.push_str(text),
                                LlmStreamDelta::Reasoning(_) => has_reasoning = true,
                            }
                            on_delta(delta)?;
                        }
                    }
                }
            }
            if content.is_empty() && has_reasoning {
                return Err(AppError::Llm("模型仅返回思考内容，尚未生成回答正文；可能已达到输出上限，请调整模型的思考预算或缩短问题。".into()));
            }
            if content.is_empty() {
                return Err(AppError::Llm("大模型服务未返回可用回答".into()));
            }
            Ok(LlmChatResponse {
                content,
                model,
                prompt_tokens: usage.as_ref().and_then(|usage: &Usage| usage.prompt_tokens),
                completion_tokens: usage
                    .as_ref()
                    .and_then(|usage: &Usage| usage.completion_tokens),
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
            let mut request = self
                .client
                .get(endpoint)
                .timeout(Duration::from_secs(config.timeout_seconds.min(15)));
            if !config.api_key.trim().is_empty() {
                request = request.bearer_auth(config.api_key.trim());
            }
            let response = request
                .send()
                .await
                .map_err(|error| request_error(error, "无法连接大模型服务"))?;
            let status = response.status();
            if status.is_success() {
                return Ok(LlmConnectionStatus {
                    provider: self.id().into(),
                    message: "OpenAI 兼容大模型服务连接正常".into(),
                });
            }
            Err(AppError::Llm(format!(
                "大模型服务连接测试返回 HTTP {status}"
            )))
        })
    }
}

fn endpoint(config: &LlmConfig, suffix: &str) -> AppResult<String> {
    let base = config.base_url.trim_end_matches('/');
    let suffix = if base.ends_with("/v1") {
        suffix.trim_start_matches("/v1")
    } else {
        suffix
    };
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
    if chars.next().is_none() {
        trimmed.into()
    } else {
        format!("{head}…")
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
    enable_thinking: bool,
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
    #[serde(default, alias = "reasoning")]
    reasoning_content: Option<String>,
}

impl StreamChatResponseDelta {
    fn into_events(self) -> impl Iterator<Item = LlmStreamDelta> {
        self.reasoning_content
            .filter(|text| !text.is_empty())
            .map(LlmStreamDelta::Reasoning)
            .into_iter()
            .chain(
                self.content
                    .filter(|text| !text.is_empty())
                    .map(LlmStreamDelta::Content),
            )
    }
}

#[derive(Clone, Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_server_errors_before_returning_them_to_the_ui() {
        assert!(response_error(&"a".repeat(501)).ends_with('…'));
    }

    #[test]
    fn forwards_reasoning_before_answer_in_the_same_chunk() {
        let event: StreamChatCompletionResponse = serde_json::from_str(
            r#"{"choices":[{"delta":{"reasoning_content":"分析", "content":"回答"}}]}"#,
        )
        .unwrap();
        let events: Vec<_> = event
            .choices
            .into_iter()
            .next()
            .unwrap()
            .delta
            .into_events()
            .collect();
        assert!(matches!(&events[0], LlmStreamDelta::Reasoning(text) if text == "分析"));
        assert!(matches!(&events[1], LlmStreamDelta::Content(text) if text == "回答"));
    }

    #[test]
    fn handles_reasoning_alias_and_empty_role_deltas() {
        let delta: StreamChatResponseDelta =
            serde_json::from_str(r#"{"reasoning":"分析","content":null}"#).unwrap();
        assert!(
            matches!(delta.into_events().next(), Some(LlmStreamDelta::Reasoning(text)) if text == "分析")
        );
        let delta: StreamChatResponseDelta =
            serde_json::from_str(r#"{"role":"assistant","content":""}"#).unwrap();
        assert_eq!(delta.into_events().count(), 0);
    }

    #[test]
    fn delivers_reasoning_while_server_is_still_generating() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            sync::mpsc,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (received_tx, received_rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            let headers = String::from_utf8(request).unwrap();
            let length: usize = headers
                .lines()
                .find_map(|line| {
                    let (key, value) = line.split_once(':')?;
                    key.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse().unwrap())
                })
                .unwrap();
            socket.read_exact(&mut vec![0; length]).unwrap();
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").unwrap();
            let mut send = |data: &str| {
                write!(socket, "{:x}\r\n{}\r\n", data.len(), data).unwrap();
                socket.flush().unwrap();
            };
            send("data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"分析\"}}]}\n\n");
            // Do not send an answer until the client has delivered reasoning.
            received_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            send("data: {\"choices\":[{\"delta\":{\"content\":\"回答\"}}]}\n\n");
            send("data: [DONE]\n\n");
            socket.write_all(b"0\r\n\r\n").unwrap();
        });
        let client = OpenAiCompatibleLlmClient::new().unwrap();
        let config = LlmConfig {
            base_url: format!("http://{address}"),
            model: "test".into(),
            api_key: String::new(),
            timeout_seconds: 10,
        };
        let request = LlmChatRequest {
            messages: vec![],
            temperature: 0.5,
        };
        let mut deltas = Vec::new();
        let result =
            tauri::async_runtime::block_on(client.chat_stream(&config, request, &mut |delta| {
                if matches!(delta, LlmStreamDelta::Reasoning(_)) {
                    received_tx.send(()).unwrap();
                }
                deltas.push(delta);
                Ok(())
            }))
            .unwrap();
        server.join().unwrap();
        assert_eq!(result.content, "回答");
        assert_eq!(deltas.len(), 2);
    }
}
