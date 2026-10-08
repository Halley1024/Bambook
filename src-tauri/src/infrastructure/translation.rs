use std::{future::Future, pin::Pin, time::Duration};
use serde::{Deserialize, Serialize};
use crate::{
    application::ports::TranslationProvider,
    domain::translation::{TranslationConfig, TranslationRequest, TranslationResult},
    error::{AppError, AppResult},
};

pub(crate) struct BaiduTranslationProvider { client: reqwest::Client }

impl BaiduTranslationProvider {
    pub(crate) fn new() -> AppResult<Self> {
        let client = reqwest::Client::builder().connect_timeout(Duration::from_secs(8)).timeout(Duration::from_secs(30))
            .build().map_err(|error| AppError::Translation(format!("无法创建网络客户端：{error}")))?;
        Ok(Self { client })
    }
}

impl TranslationProvider for BaiduTranslationProvider {
    fn id(&self) -> &'static str { "baidu" }

    fn translate<'a>(&'a self, config: &'a TranslationConfig, request: TranslationRequest)
        -> Pin<Box<dyn Future<Output = AppResult<TranslationResult>> + Send + 'a>> {
        Box::pin(async move {
            let text = request.text.trim();
            if text.is_empty() { return Err(AppError::Translation("请选择需要翻译的文字".into())); }
            if text.as_bytes().len() > 6_000 { return Err(AppError::Translation("百度翻译单次请求不能超过 6000 字节，请缩短选区".into())); }
            let salt = unique_salt();
            let app_id = config.app_id.trim();
            let secret_key = config.secret_key.trim();
            let sign = baidu_sign(app_id, text, &salt, secret_key);
            let form = BaiduTranslateForm { q: text, from: baidu_language(&request.source_language), to: baidu_language(&request.target_language),
                appid: app_id, salt: &salt, sign: &sign };
            let response = self.client.post(config.api_url.trim()).form(&form).send().await
                .map_err(|error| AppError::Translation(if error.is_timeout() { "百度翻译请求超时".into() } else { format!("无法连接百度翻译：{error}") }))?;
            let status = response.status();
            let body = response.text().await.map_err(|error| AppError::Translation(format!("无法读取百度翻译响应：{error}")))?;
            if !status.is_success() { return Err(AppError::Translation(format!("百度翻译返回 HTTP {status}"))); }
            let payload: BaiduTranslateResponse = serde_json::from_str(&body)
                .map_err(|error| AppError::Translation(format!("无法解析百度翻译响应：{error}")))?;
            if let Some(code) = payload.error_code { return Err(AppError::Translation(baidu_error(&code, payload.error_msg.as_deref()))); }
            let items = payload.trans_result.ok_or_else(|| AppError::Translation("百度翻译未返回译文".into()))?;
            let translated_text = items.into_iter().map(|item| item.dst).collect::<Vec<_>>().join("\n");
            Ok(TranslationResult { translated_text, detected_language: payload.from, character_count: text.chars().count(), usage_count: 0, provider: self.id().into() })
        })
    }
}

#[derive(Serialize)]
struct BaiduTranslateForm<'a> { q: &'a str, from: &'a str, to: &'a str, appid: &'a str, salt: &'a str, sign: &'a str }

#[derive(Deserialize)]
struct BaiduTranslateResponse { from: Option<String>, trans_result: Option<Vec<BaiduTranslationItem>>, error_code: Option<String>, error_msg: Option<String> }
#[derive(Deserialize)]
struct BaiduTranslationItem { dst: String }

fn unique_salt() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos().to_string()
}

fn baidu_sign(app_id: &str, text: &str, salt: &str, secret_key: &str) -> String {
    format!("{:x}", md5::compute(format!("{app_id}{text}{salt}{secret_key}")))
}

fn baidu_language(language: &str) -> &str {
    match language { "ja" => "jp", "ko" => "kor", "fr" => "fra", "es" => "spa", "ar" => "ara", "vi" => "vie", value => value }
}

fn baidu_error(code: &str, fallback: Option<&str>) -> String {
    let message = match code {
        "52001" => "请求超时", "52002" => "百度翻译系统错误", "52003" => "APP ID 未授权或尚未开通通用文本翻译",
        "54000" => "请求参数不完整", "54001" => "签名错误，请检查 APP ID 和百度密钥", "54003" => "访问频率受限，请稍后重试",
        "54004" => "账户余额不足", "54005" => "长文本请求过于频繁", "58000" => "当前客户端 IP 不在允许列表",
        "58001" => "目标语言不受当前账户版本支持", _ => fallback.unwrap_or("未知错误"),
    };
    format!("百度错误 {code}：{message}")
}

#[cfg(test)]
mod tests {
    use super::{baidu_error, baidu_language, baidu_sign};
    #[test] fn maps_baidu_language_codes() { assert_eq!(baidu_language("ja"), "jp"); assert_eq!(baidu_language("zh"), "zh"); }
    #[test] fn explains_invalid_signature() { assert!(baidu_error("54001", None).contains("签名错误")); }
    #[test]
    fn signs_the_official_baidu_example() {
        assert_eq!(baidu_sign("2015063000000001", "apple", "65478", "1234567890"), "a1a7461d92e5194c5cae3182b5b24de1");
    }

    #[test]
    fn credential_whitespace_can_be_removed_before_signing() {
        assert_eq!(
            baidu_sign(" 2015063000000001 ".trim(), "apple", "65478", " 1234567890\r\n".trim()),
            "a1a7461d92e5194c5cae3182b5b24de1"
        );
    }
}
