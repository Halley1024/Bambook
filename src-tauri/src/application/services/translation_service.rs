use std::{collections::HashMap, sync::{Arc, atomic::{AtomicU64, Ordering}}};
use crate::{
    application::ports::TranslationProvider,
    domain::translation::{TranslationConfig, TranslationConnectionStatus, TranslationRequest, TranslationResult},
    error::{AppError, AppResult},
};

pub(crate) struct TranslationService {
    providers: HashMap<&'static str, Arc<dyn TranslationProvider>>,
    usage_count: AtomicU64,
}

impl TranslationService {
    pub(crate) fn new(providers: Vec<Arc<dyn TranslationProvider>>) -> Self {
        Self { providers: providers.into_iter().map(|provider| (provider.id(), provider)).collect(), usage_count: AtomicU64::new(0) }
    }

    pub(crate) async fn translate(&self, config: &TranslationConfig, request: TranslationRequest) -> AppResult<TranslationResult> {
        validate_config(config)?;
        if !config.allow_third_party { return Err(AppError::Translation("请先在设置中心允许发送选中文字到第三方翻译服务".into())); }
        let provider = self.providers.get(config.provider.as_str())
            .ok_or_else(|| AppError::Translation(format!("尚未接入翻译服务商：{}", config.provider)))?;
        let mut result = provider.translate(config, request).await?;
        result.usage_count = self.usage_count.fetch_add(1, Ordering::Relaxed) + 1;
        Ok(result)
    }

    pub(crate) async fn test_connection(&self, config: &TranslationConfig) -> AppResult<TranslationConnectionStatus> {
        validate_config(config)?;
        let provider = self.providers.get(config.provider.as_str())
            .ok_or_else(|| AppError::Translation(format!("尚未接入翻译服务商：{}", config.provider)))?;
        provider.translate(config, TranslationRequest { text: "apple".into(), source_language: "en".into(), target_language: "zh".into() }).await?;
        Ok(TranslationConnectionStatus { provider: provider.id().into(), message: "百度翻译连接与鉴权测试通过".into() })
    }
}

fn validate_config(config: &TranslationConfig) -> AppResult<()> {
    if config.provider.trim().is_empty() { return Err(AppError::Translation("请选择翻译服务商".into())); }
    if config.api_url.trim().is_empty() { return Err(AppError::Translation("请填写翻译 API 地址".into())); }
    if config.app_id.trim().is_empty() { return Err(AppError::Translation("请填写百度通用文本翻译的 APP ID".into())); }
    if config.secret_key.trim().is_empty() { return Err(AppError::Translation("请填写百度通用文本翻译开发者信息中的密钥（不是大模型 API Key）".into())); }
    Ok(())
}
