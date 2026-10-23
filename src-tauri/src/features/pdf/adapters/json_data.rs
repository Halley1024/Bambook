use crate::features::library::validate_document_id;
use crate::features::library::DocumentDataLocator;
use crate::features::pdf::PdfBookmark;
use crate::features::pdf::PdfDataRepository;
use crate::features::pdf::PdfReadingState;
use crate::platform::filesystem::json_file::{read_optional_json, write_pretty_json};
use crate::shared::error::AppResult;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};

pub(crate) struct JsonPdfDataRepository {
    locations: Arc<dyn DocumentDataLocator>,
}
impl JsonPdfDataRepository {
    pub(crate) fn new(locations: Arc<dyn DocumentDataLocator>) -> Self {
        Self { locations }
    }
}
impl PdfDataRepository for JsonPdfDataRepository {
    fn load_pdf_bookmarks(
        &self,
        document_id: &str,
        source_path: Option<&str>,
    ) -> AppResult<Vec<PdfBookmark>> {
        validate_document_id(document_id)?;
        let path =
            self.locations
                .companion_read_path(document_id, source_path, "bookmarks.json")?;
        let stored: Option<Vec<StoredPdfBookmark>> = match path {
            Some(path) => read_optional_json(&path)?,
            None => None,
        };
        Ok(stored
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn save_pdf_bookmarks(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        bookmarks: &[PdfBookmark],
    ) -> AppResult<()> {
        validate_document_id(document_id)?;
        let stored: Vec<StoredPdfBookmark> = bookmarks.iter().map(Into::into).collect();
        if let Some(directory) = self
            .locations
            .persisted_document_data_directory(document_id)?
        {
            write_pretty_json(&directory.join("bookmarks.json"), &stored)?;
        }
        if let Some(sidecar) = existing_sidecar_path(source_path, "bookmarks.json") {
            write_pretty_json(&sidecar, &stored)?;
        }
        Ok(())
    }

    fn load_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
    ) -> AppResult<Option<PdfReadingState>> {
        validate_document_id(document_id)?;
        let path =
            self.locations
                .companion_read_path(document_id, source_path, "reading-state.json")?;
        let stored: Option<StoredPdfReadingState> = match path {
            Some(path) => read_optional_json(&path)?,
            None => None,
        };
        Ok(stored.map(Into::into))
    }

    fn save_pdf_reading_state(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        state: &PdfReadingState,
    ) -> AppResult<()> {
        validate_document_id(document_id)?;
        let stored = StoredPdfReadingState::from(state);
        if let Some(directory) = self
            .locations
            .persisted_document_data_directory(document_id)?
        {
            write_pretty_json(&directory.join("reading-state.json"), &stored)?;
        }
        if let Some(sidecar) = existing_sidecar_path(source_path, "reading-state.json") {
            write_pretty_json(&sidecar, &stored)?;
        }
        Ok(())
    }
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredPdfBookmark {
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

impl From<StoredPdfBookmark> for PdfBookmark {
    fn from(value: StoredPdfBookmark) -> Self {
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

impl From<&PdfBookmark> for StoredPdfBookmark {
    fn from(value: &PdfBookmark) -> Self {
        Self {
            id: value.id.clone(),
            document_id: value.document_id.clone(),
            page: value.page,
            x: value.x,
            y: value.y,
            title: value.title.clone(),
            color: value.color.clone(),
            created_at: value.created_at.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredPdfReadingState {
    page: u32,
    page_y: Option<f64>,
    zoom_mode: String,
    scale: f64,
    updated_at: String,
}

impl From<StoredPdfReadingState> for PdfReadingState {
    fn from(value: StoredPdfReadingState) -> Self {
        Self {
            page: value.page,
            page_y: value.page_y,
            zoom_mode: value.zoom_mode,
            scale: value.scale,
            updated_at: value.updated_at,
        }
    }
}

impl From<&PdfReadingState> for StoredPdfReadingState {
    fn from(value: &PdfReadingState) -> Self {
        Self {
            page: value.page,
            page_y: value.page_y,
            zoom_mode: value.zoom_mode.clone(),
            scale: value.scale,
            updated_at: value.updated_at.clone(),
        }
    }
}

fn default_bookmark_color() -> String {
    "#26765A".into()
}

fn existing_sidecar_path(source_path: Option<&str>, file_name: &str) -> Option<std::path::PathBuf> {
    source_path
        .and_then(|path| Path::new(path).parent())
        .map(|parent| parent.join(file_name))
        .filter(|path| path.is_file())
}
