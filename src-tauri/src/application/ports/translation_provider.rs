use std::{future::Future, pin::Pin};
use crate::{domain::translation::{TranslationConfig, TranslationRequest, TranslationResult}, error::AppResult};

pub(crate) trait TranslationProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn translate<'a>(&'a self, config: &'a TranslationConfig, request: TranslationRequest)
        -> Pin<Box<dyn Future<Output = AppResult<TranslationResult>> + Send + 'a>>;
}
