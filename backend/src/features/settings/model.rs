#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum StartupBehavior {
    #[default]
    RestoreWorkspace,
    NewMarkdown,
    StartPage,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum DocumentImportMode {
    #[default]
    Copy,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkspacePanelLayout {
    pub(crate) left_width: u32,  // 左侧面板宽度。
    pub(crate) right_width: u32, // 右侧面板宽度。
}

impl WorkspacePanelLayout {
    pub(crate) const fn pdf_default() -> Self {
        Self {
            left_width: 280,
            right_width: 340,
        }
    }

    pub(crate) const fn markdown_default() -> Self {
        Self {
            left_width: 280,
            right_width: 310,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AppSettings {
    pub(crate) annotation_storage_dir: Option<String>, // 自定义批注存储目录；为空时使用默认目录。
    pub(crate) export_dir: Option<String>,             // 自定义导出目录；为空时使用默认目录。
    pub(crate) pdf_storage_dir: Option<String>,        // 自定义 PDF 存储目录；为空时使用默认目录。
    pub(crate) markdown_storage_dir: Option<String>, // 自定义 Markdown 存储目录；为空时使用默认目录。
    pub(crate) pdf_import_mode: DocumentImportMode,  // 导入 PDF 时采用复制还是链接。
    pub(crate) markdown_import_mode: DocumentImportMode, // 导入 Markdown 时采用复制还是链接。
    pub(crate) theme: ThemePreference,               // 应用主题偏好。
    pub(crate) animations_enabled: bool,             // 是否启用界面动画。
    pub(crate) confirmation_reminders_enabled: bool, // 是否启用操作确认提醒。
    pub(crate) startup_behavior: StartupBehavior,    // 应用启动后的初始行为。
    pub(crate) pdf_workspace_layout: WorkspacePanelLayout, // PDF 工作区的左右面板宽度。
    pub(crate) markdown_workspace_layout: WorkspacePanelLayout, // Markdown 工作区的左右面板宽度。
    pub(crate) translation_provider: String,         // 翻译服务提供方标识。
    pub(crate) translation_api_url: String,          // 翻译服务接口地址。
    pub(crate) translation_api_key: String,          // 翻译服务的 API 密钥或应用标识。
    pub(crate) translation_secret_id: String,        // 翻译服务的身份标识。
    pub(crate) translation_secret_key: String,       // 翻译服务的身份密钥。
    pub(crate) translation_target_language: String,  // 默认翻译目标语言。
    pub(crate) allow_third_party_translation: bool,  // 是否允许使用第三方翻译服务。
    pub(crate) llm_base_url: String,                 // LLM 服务根地址。
    pub(crate) llm_model: String,                    // 默认使用的 LLM 模型名称。
    pub(crate) llm_api_key: String,                  // LLM 服务的 API 密钥。
    pub(crate) llm_timeout_seconds: u64,             // LLM 请求超时秒数。
    pub(crate) llm_temperature: f32,                 // LLM 生成温度。
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            annotation_storage_dir: None,
            export_dir: None,
            pdf_storage_dir: None,
            markdown_storage_dir: None,
            pdf_import_mode: DocumentImportMode::Copy,
            markdown_import_mode: DocumentImportMode::Copy,
            theme: ThemePreference::System,
            animations_enabled: true,
            confirmation_reminders_enabled: true,
            startup_behavior: StartupBehavior::RestoreWorkspace,
            pdf_workspace_layout: WorkspacePanelLayout::pdf_default(),
            markdown_workspace_layout: WorkspacePanelLayout::markdown_default(),
            translation_provider: "baidu".into(),
            translation_api_url: "https://fanyi-api.baidu.com/api/trans/vip/translate".into(),
            translation_api_key: String::new(),
            translation_secret_id: String::new(),
            translation_secret_key: String::new(),
            translation_target_language: "zh".into(),
            allow_third_party_translation: false,
            llm_base_url: "http://127.0.0.1:8080".into(),
            llm_model: "local-model".into(),
            llm_api_key: String::new(),
            llm_timeout_seconds: 120,
            llm_temperature: 0.2,
        }
    }
}
