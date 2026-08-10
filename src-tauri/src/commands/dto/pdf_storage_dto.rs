use serde::{Deserialize, Serialize};

use crate::domain::storage::{PdfBookmark, PdfReadingState};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PdfBookmarkDto {
    id: String,
    document_id: String,
    page: u32,
    x: f64,
    y: f64,
    title: String,
    #[serde(default = "default_bookmark_color")]
    color: String,
    created_at: String,
}

impl From<PdfBookmarkDto> for PdfBookmark {
    fn from(value: PdfBookmarkDto) -> Self {
        Self {
            id: value.id,
            document_id: value.document_id,
            page: value.page,
            x: value.x,
            y: value.y,
            title: value.title,
            color: value.color,
            created_at: value.created_at,
        }
    }
}

impl From<PdfBookmark> for PdfBookmarkDto {
    fn from(value: PdfBookmark) -> Self {
        Self {
            id: value.id,
            document_id: value.document_id,
            page: value.page,
            x: value.x,
            y: value.y,
            title: value.title,
            color: value.color,
            created_at: value.created_at,
        }
    }
}

fn default_bookmark_color() -> String { "#26765A".into() }

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PdfReadingStateDto {
    page: u32,
    page_y: Option<f64>,
    zoom_mode: String,
    scale: f64,
    updated_at: String,
}

impl From<PdfReadingStateDto> for PdfReadingState {
    fn from(value: PdfReadingStateDto) -> Self {
        Self {
            page: value.page,
            page_y: value.page_y,
            zoom_mode: value.zoom_mode,
            scale: value.scale,
            updated_at: value.updated_at,
        }
    }
}

impl From<PdfReadingState> for PdfReadingStateDto {
    fn from(value: PdfReadingState) -> Self {
        Self {
            page: value.page,
            page_y: value.page_y,
            zoom_mode: value.zoom_mode,
            scale: value.scale,
            updated_at: value.updated_at,
        }
    }
}
