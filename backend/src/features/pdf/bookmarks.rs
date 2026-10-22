#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfBookmark {
    pub(crate) id: String,          // 书签的唯一标识。
    pub(crate) document_id: String, // 所属 PDF 文档的标识。
    pub(crate) page: u32,           // 书签所在页码。
    pub(crate) x: f64,              // 书签在页面中的水平位置。
    pub(crate) y: f64,              // 书签在页面中的垂直位置。
    pub(crate) title: String,       // 书签标题。
    pub(crate) color: String,       // 书签颜色。
    pub(crate) created_at: String,  // 创建时间的持久化文本。
}
