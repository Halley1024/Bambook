use std::sync::Arc;
use tauri::State;
use crate::{commands::dto::{TranslationConfigDto, TranslationConnectionStatusDto, TranslationRequestDto, TranslationResultDto}, error::AppResult, state::AppState};

#[tauri::command]
pub(crate) async fn translate_text(state: State<'_, AppState>, config: TranslationConfigDto, request: TranslationRequestDto) -> AppResult<TranslationResultDto> {
    let service = Arc::clone(&state.translation);
    service.translate(&config.into(), request.into()).await.map(Into::into)
}

#[tauri::command]
pub(crate) async fn test_translation_connection(state: State<'_, AppState>, config: TranslationConfigDto) -> AppResult<TranslationConnectionStatusDto> {
    let service = Arc::clone(&state.translation);
    service.test_connection(&config.into()).await.map(Into::into)
}
