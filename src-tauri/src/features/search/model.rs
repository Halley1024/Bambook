#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchOptions {
    pub(crate) case_sensitive: bool, // 是否区分字母大小写。
    pub(crate) whole_word: bool,     // 是否只匹配完整单词。
    pub(crate) result_limit: usize,  // 最多返回的搜索结果数。
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            result_limit: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SearchTarget {
    Pdf {
        session_id: String, // 待搜索 PDF 的会话标识。
    },
    Markdown {
        session_id: String, // 待搜索 Markdown 的会话标识。
        revision: u64,      // 待搜索的 Markdown 版本号。
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchRequest {
    pub(crate) request_id: u64,        // 用于识别及取消本次搜索的请求标识。
    pub(crate) target: SearchTarget,   // 待搜索的文档及其版本。
    pub(crate) query: String,          // 搜索关键词。
    pub(crate) options: SearchOptions, // 搜索匹配选项。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SearchResponse {
    pub(crate) request_id: u64,           // 对应搜索请求的标识。
    pub(crate) total: usize,              // 实际找到的结果数量。
    pub(crate) truncated: bool,           // 结果是否因数量限制而截断。
    pub(crate) cancelled: bool,           // 搜索是否已被取消。
    pub(crate) matches: Vec<SearchMatch>, // 返回的搜索命中项。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SearchMatch {
    pub(crate) id: String,             // 命中项的标识。
    pub(crate) matched_text: String,   // 命中的原文。
    pub(crate) before_text: String,    // 命中位置前的上下文。
    pub(crate) after_text: String,     // 命中位置后的上下文。
    pub(crate) location_label: String, // 用于展示命中位置的标签。
    pub(crate) locator: SearchLocator, // 文档类型对应的精确定位信息。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SearchLocator {
    Pdf(PdfSearchLocator),
    Markdown(MarkdownSearchLocator),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfSearchLocator {
    pub(crate) page: u32,              // 命中项所在的 PDF 页索引。
    pub(crate) page_y: f32,            // 命中项在页面中的垂直位置。
    pub(crate) rects: Vec<SearchRect>, // 命中文字覆盖的页面矩形。
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownSearchLocator {
    pub(crate) revision: u64,      // 命中内容所属的 Markdown 版本号。
    pub(crate) start_utf16: usize, // 命中范围起点的 UTF-16 偏移。
    pub(crate) end_utf16: usize,   // 命中范围终点的 UTF-16 偏移。
    pub(crate) line: usize,        // 命中位置所在的行号。
    pub(crate) column: usize,      // 命中位置所在的列号。
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SearchRect {
    pub(crate) left: f32,   // 矩形左边界坐标。
    pub(crate) top: f32,    // 矩形上边界坐标。
    pub(crate) width: f32,  // 矩形宽度。
    pub(crate) height: f32, // 矩形高度。
}
