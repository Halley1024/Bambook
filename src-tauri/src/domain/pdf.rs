#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfDocumentModel {
    pub(crate) document_id: String,
    pub(crate) session_id: String,
    pub(crate) path: String,
    pub(crate) source_path: String,
    pub(crate) title: String,
    pub(crate) page_count: u32,
    pub(crate) needs_password: bool,
    pub(crate) metadata: PdfMetadata,
    pub(crate) outline: Vec<PdfOutlineNode>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PdfMetadata {
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) subject: String,
    pub(crate) keywords: String,
    pub(crate) creator: String,
    pub(crate) producer: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfOutlineNode {
    pub(crate) title: String,
    pub(crate) page: Option<u32>,
    pub(crate) x: Option<f32>,
    pub(crate) y: Option<f32>,
    pub(crate) uri: Option<String>,
    pub(crate) children: Vec<PdfOutlineNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfPageStructure {
    pub(crate) page_index: u32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) blocks: Vec<PdfTextBlock>,
    pub(crate) links: Vec<PdfLink>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextBlock {
    pub(crate) bounds: PdfRect,
    pub(crate) lines: Vec<PdfTextLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextLine {
    pub(crate) bounds: PdfRect,
    pub(crate) spans: Vec<PdfTextSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfTextSpan {
    pub(crate) text: String,
    pub(crate) bounds: PdfRect,
    pub(crate) font_size: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PdfLink {
    pub(crate) bounds: PdfRect,
    pub(crate) uri: String,
    pub(crate) target_page: Option<u32>,
    pub(crate) target_x: Option<f32>,
    pub(crate) target_y: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PdfRect {
    pub(crate) left: f32,
    pub(crate) top: f32,
    pub(crate) right: f32,
    pub(crate) bottom: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderedPdfPage {
    pub(crate) png: Vec<u8>,
}
