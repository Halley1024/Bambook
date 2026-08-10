#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchOptions {
    pub(crate) case_sensitive: bool,
    pub(crate) whole_word: bool,
    pub(crate) result_limit: usize,
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
    Pdf { session_id: String },
    Markdown { session_id: String, revision: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchRequest {
    pub(crate) request_id: u64,
    pub(crate) target: SearchTarget,
    pub(crate) query: String,
    pub(crate) options: SearchOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SearchResponse {
    pub(crate) request_id: u64,
    pub(crate) total: usize,
    pub(crate) truncated: bool,
    pub(crate) cancelled: bool,
    pub(crate) matches: Vec<SearchMatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SearchMatch {
    pub(crate) id: String,
    pub(crate) matched_text: String,
    pub(crate) before_text: String,
    pub(crate) after_text: String,
    pub(crate) location_label: String,
    pub(crate) locator: SearchLocator,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SearchLocator {
    Pdf(PdfSearchLocator),
    Markdown(MarkdownSearchLocator),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfSearchLocator {
    pub(crate) page: u32,
    pub(crate) page_y: f32,
    pub(crate) rects: Vec<SearchRect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownSearchLocator {
    pub(crate) revision: u64,
    pub(crate) start_utf16: usize,
    pub(crate) end_utf16: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SearchRect {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}
