use std::sync::Arc;

use tauri::{ipc::Channel, State};

use crate::{
    commands::dto::{LlmChatRequestDto, LlmChatResponseDto, LlmConfigDto, LlmConnectionStatusDto, LlmStreamEventDto},
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn ask_llm(
    state: State<'_, AppState>,
    config: LlmConfigDto,
    request: LlmChatRequestDto,
    on_event: Channel<LlmStreamEventDto>,
) -> AppResult<LlmChatResponseDto> {
    let service = Arc::clone(&state.llm);
    let result = service
        .chat_stream(&config.into(), request.into(), &mut |content| {
            on_event.send(LlmStreamEventDto::Delta { content })
                .map_err(|error| AppError::Llm(format!("无法发送流式回答：{error}")))
        })
        .await;
    if result.is_ok() {
        on_event.send(LlmStreamEventDto::Complete)
            .map_err(|error| AppError::Llm(format!("无法发送流式完成状态：{error}")))?;
    }
    result.map(Into::into)
}

#[tauri::command]
pub(crate) async fn test_llm_connection(
    state: State<'_, AppState>,
    config: LlmConfigDto,
) -> AppResult<LlmConnectionStatusDto> {
    let service = Arc::clone(&state.llm);
    service.test_connection(&config.into()).await.map(Into::into)
}
