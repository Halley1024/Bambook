use serde::Serialize;

use crate::domain::pdf::{
    PdfDocumentModel, PdfLink, PdfMetadata, PdfOutlineNode, PdfPageStructure, PdfRect,
    PdfTextBlock, PdfTextLine, PdfTextSpan,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PdfDocumentDto {
    document_id: String,
    session_id: String,
    path: String,
    source_path: String,
    title: String,
    page_count: u32,
    needs_password: bool,
    metadata: PdfMetadataDto,
    outline: Vec<PdfOutlineNodeDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfMetadataDto {
    title: String,
    author: String,
    subject: String,
    keywords: String,
    creator: String,
    producer: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfOutlineNodeDto {
    title: String,
    page: Option<u32>,
    x: Option<f32>,
    y: Option<f32>,
    uri: Option<String>,
    children: Vec<PdfOutlineNodeDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PdfPageStructureDto {
    page_index: u32,
    width: f32,
    height: f32,
    blocks: Vec<PdfTextBlockDto>,
    links: Vec<PdfLinkDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfTextBlockDto {
    bounds: PdfRectDto,
    lines: Vec<PdfTextLineDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfTextLineDto {
    bounds: PdfRectDto,
    spans: Vec<PdfTextSpanDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfTextSpanDto {
    text: String,
    bounds: PdfRectDto,
    font_size: f32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfLinkDto {
    bounds: PdfRectDto,
    uri: String,
    target_page: Option<u32>,
    target_x: Option<f32>,
    target_y: Option<f32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfRectDto {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl From<PdfDocumentModel> for PdfDocumentDto {
    fn from(value: PdfDocumentModel) -> Self {
        Self {
            document_id: value.document_id,
            session_id: value.session_id,
            path: value.path,
            source_path: value.source_path,
            title: value.title,
            page_count: value.page_count,
            needs_password: value.needs_password,
            metadata: value.metadata.into(),
            outline: value.outline.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<PdfMetadata> for PdfMetadataDto {
    fn from(v: PdfMetadata) -> Self {
        Self {
            title: v.title,
            author: v.author,
            subject: v.subject,
            keywords: v.keywords,
            creator: v.creator,
            producer: v.producer,
        }
    }
}
impl From<PdfOutlineNode> for PdfOutlineNodeDto {
    fn from(v: PdfOutlineNode) -> Self {
        Self {
            title: v.title,
            page: v.page,
            x: v.x,
            y: v.y,
            uri: v.uri,
            children: v.children.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<PdfPageStructure> for PdfPageStructureDto {
    fn from(v: PdfPageStructure) -> Self {
        Self {
            page_index: v.page_index,
            width: v.width,
            height: v.height,
            blocks: v.blocks.into_iter().map(Into::into).collect(),
            links: v.links.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<PdfTextBlock> for PdfTextBlockDto {
    fn from(v: PdfTextBlock) -> Self {
        Self {
            bounds: v.bounds.into(),
            lines: v.lines.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<PdfTextLine> for PdfTextLineDto {
    fn from(v: PdfTextLine) -> Self {
        Self {
            bounds: v.bounds.into(),
            spans: v.spans.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<PdfTextSpan> for PdfTextSpanDto {
    fn from(v: PdfTextSpan) -> Self {
        Self {
            text: v.text,
            bounds: v.bounds.into(),
            font_size: v.font_size,
        }
    }
}
impl From<PdfLink> for PdfLinkDto {
    fn from(v: PdfLink) -> Self {
        Self {
            bounds: v.bounds.into(),
            uri: v.uri,
            target_page: v.target_page,
            target_x: v.target_x,
            target_y: v.target_y,
        }
    }
}
impl From<PdfRect> for PdfRectDto {
    fn from(v: PdfRect) -> Self {
        Self {
            left: v.left,
            top: v.top,
            right: v.right,
            bottom: v.bottom,
        }
    }
}
