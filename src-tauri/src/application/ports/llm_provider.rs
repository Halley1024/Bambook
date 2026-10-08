use std::{future::Future, pin::Pin};

use crate::{
    domain::llm::{LlmChatRequest, LlmChatResponse, LlmConfig, LlmConnectionStatus},
    error::AppResult,
};

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
