use std::{future::Future, pin::Pin};

use crate::features::assistant::llm::LlmChatRequest;
use crate::features::assistant::llm::LlmChatResponse;
use crate::features::assistant::llm::LlmConfig;
use crate::features::assistant::llm::LlmConnectionStatus;
use crate::shared::error::AppResult;

pub(crate) trait LlmClient: Send + Sync {
    fn id(&self) -> &'static str;
    fn chat_stream<'a>(
        &'a self,
        config: &'a LlmConfig,
        request: LlmChatRequest,
        on_delta: &'a mut (dyn FnMut(String) -> AppResult<()> + Send),
    ) -> Pin<Box<dyn Future<Output = AppResult<LlmChatResponse>> + Send + 'a>>;
    fn test_connection<'a>(
        &'a self,
        config: &'a LlmConfig,
    ) -> Pin<Box<dyn Future<Output = AppResult<LlmConnectionStatus>> + Send + 'a>>;
}
