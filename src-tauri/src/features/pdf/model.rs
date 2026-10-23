#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfDocumentModel {
    pub(crate) document_id: String,          // 文档在应用存储中的标识。
    pub(crate) session_id: String,           // 当前打开的 PDF 会话标识。
    pub(crate) path: String,                 // 本次会话实际打开的文件路径。
    pub(crate) source_path: String,          // 文档原始文件的路径。
    pub(crate) title: String,                // 文档的显示标题。
    pub(crate) page_count: u32,              // 文档总页数。
    pub(crate) needs_password: bool,         // 是否需要密码才能读取文档内容。
    pub(crate) metadata: PdfMetadata,        // PDF 元数据。
    pub(crate) outline: Vec<PdfOutlineNode>, // PDF 目录节点。
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PdfMetadata {
    pub(crate) title: String,    // PDF 元数据中的标题。
    pub(crate) author: String,   // PDF 元数据中的作者。
    pub(crate) subject: String,  // PDF 元数据中的主题。
    pub(crate) keywords: String, // PDF 元数据中的关键词。
    pub(crate) creator: String,  // 创建 PDF 的应用程序。
    pub(crate) producer: String, // 生成 PDF 的软件或组件。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfOutlineNode {
    pub(crate) title: String,                 // 目录项标题。
    pub(crate) page: Option<u32>,             // 目录项指向的页码。
    pub(crate) x: Option<f32>,                // 目标页中的水平位置。
    pub(crate) y: Option<f32>,                // 目标页中的垂直位置。
    pub(crate) uri: Option<String>,           // 目录项关联的目标 URI。
    pub(crate) children: Vec<PdfOutlineNode>, // 下级目录项。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfPageStructure {
    pub(crate) page_index: u32,           // 页面在文档中的零基索引。
    pub(crate) width: f32,                // 页面宽度，以 PDF 页面坐标表示。
    pub(crate) height: f32,               // 页面高度，以 PDF 页面坐标表示。
    pub(crate) blocks: Vec<PdfTextBlock>, // 页面中的文字块。
    pub(crate) links: Vec<PdfLink>,       // 页面中的链接区域。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextBlock {
    pub(crate) bounds: PdfRect,         // 文字块的边界矩形。
    pub(crate) lines: Vec<PdfTextLine>, // 文字块中的文本行。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextLine {
    pub(crate) bounds: PdfRect,         // 文本行的边界矩形。
    pub(crate) spans: Vec<PdfTextSpan>, // 文本行中的文字片段。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextSpan {
    pub(crate) text: String,    // 文字片段的内容。
    pub(crate) bounds: PdfRect, // 文字片段的边界矩形。
    pub(crate) font_size: f32,  // 文字片段的字号。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfLink {
    pub(crate) bounds: PdfRect,          // 可点击区域的边界矩形。
    pub(crate) uri: String,              // 链接目标 URI。
    pub(crate) target_page: Option<u32>, // 文档内目标页的索引。
    pub(crate) target_x: Option<f32>,    // 文档内目标页的水平位置。
    pub(crate) target_y: Option<f32>,    // 文档内目标页的垂直位置。
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PdfRect {
    pub(crate) left: f32,   // 矩形左边界坐标。
    pub(crate) top: f32,    // 矩形上边界坐标。
    pub(crate) right: f32,  // 矩形右边界坐标。
    pub(crate) bottom: f32, // 矩形下边界坐标。
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderedPdfPage {
    pub(crate) png: Vec<u8>, // 渲染后的 PNG 图像字节。
}
