use crate::features::assistant::translation::TranslationConfigDto;
use crate::features::assistant::translation::TranslationConnectionStatusDto;
use crate::features::assistant::translation::TranslationRequestDto;
use crate::features::assistant::translation::TranslationResultDto;
use crate::features::assistant::translation::TranslationService;
use crate::shared::error::AppResult;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub(crate) async fn translate_text(
    state: State<'_, Arc<TranslationService>>,
    config: TranslationConfigDto,
    request: TranslationRequestDto,
) -> AppResult<TranslationResultDto> {
    let service = Arc::clone(state.inner());
    service
        .translate(&config.into(), request.into())
        .await
        .map(Into::into)
}

#[tauri::command]
pub(crate) async fn test_translation_connection(
    state: State<'_, Arc<TranslationService>>,
    config: TranslationConfigDto,
) -> AppResult<TranslationConnectionStatusDto> {
    let service = Arc::clone(state.inner());
    service
        .test_connection(&config.into())
        .await
        .map(Into::into)
}
