use crate::features::assistant::translation::TranslationConfig;
use crate::features::assistant::translation::TranslationRequest;
use crate::features::assistant::translation::TranslationResult;
use crate::shared::error::AppResult;
use std::{future::Future, pin::Pin};

pub(crate) trait TranslationProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn translate<'a>(
        &'a self,
        config: &'a TranslationConfig,
        request: TranslationRequest,
    ) -> Pin<Box<dyn Future<Output = AppResult<TranslationResult>> + Send + 'a>>;
}
