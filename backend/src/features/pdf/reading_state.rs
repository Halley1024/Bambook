#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfReadingState {
    pub(crate) page: u32,           // 上次阅读的页码。
    pub(crate) page_y: Option<f64>, // 页面内的垂直阅读位置。
    pub(crate) zoom_mode: String,   // 缩放模式标识。
    pub(crate) scale: f64,          // 上次使用的缩放比例。
    pub(crate) updated_at: String,  // 最后更新时间的持久化文本。
}
