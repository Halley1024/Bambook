#[derive(Debug, Clone)]
pub(crate) struct TranslationConfig {
    pub(crate) provider: String,        // 翻译服务提供方标识。
    pub(crate) api_url: String,         // 翻译接口地址。
    pub(crate) app_id: String,          // 翻译服务的应用标识。
    pub(crate) secret_key: String,      // 翻译服务密钥。
    pub(crate) allow_third_party: bool, // 是否允许向第三方翻译服务发送文本。
}

#[derive(Debug, Clone)]
pub(crate) struct TranslationRequest {
    pub(crate) text: String,            // 待翻译的原文。
    pub(crate) source_language: String, // 原文语言标识。
    pub(crate) target_language: String, // 目标语言标识。
}

#[derive(Debug, Clone)]
pub(crate) struct TranslationResult {
    pub(crate) translated_text: String,           // 翻译后的文本。
    pub(crate) detected_language: Option<String>, // 服务检测出的原文语言。
    pub(crate) character_count: usize,            // 本次请求计入用量的字符数。
    pub(crate) usage_count: u64,                  // 本次请求计入的调用次数。
    pub(crate) provider: String,                  // 实际使用的翻译服务提供方。
}

#[derive(Debug, Clone)]
pub(crate) struct TranslationConnectionStatus {
    pub(crate) provider: String, // 被测试的翻译服务提供方。
    pub(crate) message: String,  // 连接测试的结果说明。
}
