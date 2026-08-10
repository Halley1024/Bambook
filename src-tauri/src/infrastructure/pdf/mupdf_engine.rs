use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread,
};

use mupdf::{
    Colorspace, DestinationKind, Document, ImageFormat, Matrix, MetadataName, Outline, Quad, Rect, TextBlockContent,
    TextPageFlags,
};

use crate::{
    application::ports::PdfEngine,
    domain::pdf::{
        PdfDocumentModel, PdfLink, PdfMetadata, PdfOutlineNode, PdfPageStructure, PdfRect,
        PdfTextBlock, PdfTextLine, PdfTextSpan, RenderedPdfPage,
    },
    error::{AppError, AppResult},
};

pub(crate) struct MupdfPdfEngine {
    sessions: Mutex<HashMap<String, Sender<Request>>>,
    next_session_id: AtomicU64,
}

enum Request {
    Authenticate(String, Sender<AppResult<PdfDocumentModel>>),
    PageStructure(u32, Sender<AppResult<PdfPageStructure>>),
    Render(u32, f32, Sender<AppResult<RenderedPdfPage>>),
    Close,
}

impl Default for MupdfPdfEngine {
    fn default() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_session_id: AtomicU64::new(1),
        }
    }
}

impl PdfEngine for MupdfPdfEngine {
    fn open(&self, path: &str, password: Option<&str>) -> AppResult<PdfDocumentModel> {
        let session_id = format!(
            "pdf-{}",
            self.next_session_id.fetch_add(1, Ordering::Relaxed)
        );
        let (request_tx, request_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let worker_path = path.to_owned();
        let worker_session_id = session_id.clone();
        let worker_password = password.map(str::to_owned);

        thread::Builder::new()
            .name(format!("mupdf-{session_id}"))
            .spawn(move || {
                run_worker(
                    worker_session_id,
                    worker_path,
                    worker_password,
                    request_rx,
                    ready_tx,
                )
            })
            .map_err(|error| AppError::MuPdf(error.to_string()))?;

        let document = ready_rx.recv().map_err(|_| AppError::StateUnavailable)??;
        self.sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .insert(session_id, request_tx);
        Ok(document)
    }

    fn authenticate(&self, session_id: &str, password: &str) -> AppResult<PdfDocumentModel> {
        self.request(session_id, |reply| {
            Request::Authenticate(password.to_owned(), reply)
        })
    }

    fn page_structure(&self, session_id: &str, page_index: u32) -> AppResult<PdfPageStructure> {
        self.request(session_id, |reply| {
            Request::PageStructure(page_index, reply)
        })
    }

    fn render_page(
        &self,
        session_id: &str,
        page_index: u32,
        scale: f32,
    ) -> AppResult<RenderedPdfPage> {
        self.request(session_id, |reply| {
            Request::Render(page_index, scale, reply)
        })
    }

    fn close(&self, session_id: &str) -> AppResult<()> {
        let sender = self
            .sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .remove(session_id)
            .ok_or(AppError::PdfSessionNotFound)?;
        sender
            .send(Request::Close)
            .map_err(|_| AppError::StateUnavailable)
    }
}

impl MupdfPdfEngine {
    fn request<T>(
        &self,
        session_id: &str,
        make: impl FnOnce(Sender<AppResult<T>>) -> Request,
    ) -> AppResult<T> {
        let sender = self
            .sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .get(session_id)
            .cloned()
            .ok_or(AppError::PdfSessionNotFound)?;
        let (reply_tx, reply_rx) = mpsc::channel();
        sender
            .send(make(reply_tx))
            .map_err(|_| AppError::StateUnavailable)?;
        reply_rx.recv().map_err(|_| AppError::StateUnavailable)?
    }
}

fn run_worker(
    session_id: String,
    path: String,
    password: Option<String>,
    requests: Receiver<Request>,
    ready: Sender<AppResult<PdfDocumentModel>>,
) {
    let mut document = match Document::open(&path).map_err(mupdf_error) {
        Ok(document) => document,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };

    let encrypted = document.needs_password().unwrap_or(false);
    let mut authenticated = !encrypted;
    if encrypted {
        if let Some(password) = password {
            match document.authenticate(&password) {
                Ok(true) => authenticated = true,
                Ok(false) => {
                    let _ = ready.send(Err(AppError::PdfInvalidPassword));
                    return;
                }
                Err(error) => {
                    let _ = ready.send(Err(mupdf_error(error)));
                    return;
                }
            }
        }
    }
    let initial = describe_document(&document, &session_id, &path, authenticated);
    if ready.send(initial).is_err() {
        return;
    }

    while let Ok(request) = requests.recv() {
        match request {
            Request::Authenticate(password, reply) => {
                let result = match document.authenticate(&password) {
                    Ok(true) => {
                        authenticated = true;
                        describe_document(&document, &session_id, &path, true)
                    }
                    Ok(false) => Err(AppError::PdfInvalidPassword),
                    Err(error) => Err(mupdf_error(error)),
                };
                let _ = reply.send(result);
            }
            Request::PageStructure(index, reply) => {
                let _ = reply.send(extract_page(&document, index, authenticated));
            }
            Request::Render(index, scale, reply) => {
                let _ = reply.send(render_page(&document, index, scale, authenticated));
            }
            Request::Close => break,
        }
    }
}

fn describe_document(
    document: &Document,
    session_id: &str,
    path: &str,
    authenticated: bool,
) -> AppResult<PdfDocumentModel> {
    let needs_password = document.needs_password().map_err(mupdf_error)? && !authenticated;
    let title = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .to_owned();
    if needs_password {
        return Ok(PdfDocumentModel {
            document_id: String::new(),
            session_id: session_id.to_owned(),
            path: path.to_owned(),
            source_path: path.to_owned(),
            title,
            page_count: 0,
            needs_password: true,
            metadata: PdfMetadata::default(),
            outline: Vec::new(),
        });
    }
    let metadata = PdfMetadata {
        title: metadata(document, MetadataName::Title),
        author: metadata(document, MetadataName::Author),
        subject: metadata(document, MetadataName::Subject),
        keywords: metadata(document, MetadataName::Keywords),
        creator: metadata(document, MetadataName::Creator),
        producer: metadata(document, MetadataName::Producer),
    };
    Ok(PdfDocumentModel {
        document_id: String::new(),
        session_id: session_id.to_owned(),
        path: path.to_owned(),
        source_path: path.to_owned(),
        title: if metadata.title.is_empty() {
            title
        } else {
            metadata.title.clone()
        },
        page_count: document.page_count().map_err(mupdf_error)?.max(0) as u32,
        needs_password: false,
        metadata,
        outline: outline_nodes(document.outlines().map_err(mupdf_error)?),
    })
}

fn metadata(document: &Document, name: MetadataName) -> String {
    document.metadata(name).unwrap_or_default()
}

fn outline_nodes(values: Vec<Outline>) -> Vec<PdfOutlineNode> {
    values
        .into_iter()
        .map(|value| PdfOutlineNode {
            title: value.title,
            page: value.dest.map(|dest| dest.loc.page_number),
            x: None,
            y: None,
            uri: value.uri,
            children: outline_nodes(value.down),
        })
        .collect()
}

fn extract_page(
    document: &Document,
    page_index: u32,
    authenticated: bool,
) -> AppResult<PdfPageStructure> {
    validate_page(document, page_index, authenticated)?;
    let page = document.load_page(page_index as i32).map_err(mupdf_error)?;
    let bounds = page.bounds().map_err(mupdf_error)?;
    let structured = page
        .to_text_page(TextPageFlags::PRESERVE_WHITESPACE | TextPageFlags::PRESERVE_SPANS)
        .map_err(mupdf_error)?
        .structured();
    let blocks = structured
        .blocks
        .into_iter()
        .filter_map(|block| match block.content {
            TextBlockContent::Text { lines } => Some(PdfTextBlock {
                bounds: page_rect(block.bounds, bounds),
                lines: lines
                    .into_iter()
                    .map(|line| {
                        let spans = if line.chars.is_empty() {
                            vec![PdfTextSpan {
                                text: line.text,
                                bounds: page_rect(line.bounds, bounds),
                                font_size: line.bounds.height(),
                            }]
                        } else {
                            line.chars
                                .into_iter()
                                .map(|character| PdfTextSpan {
                                    text: character.ch.to_string(),
                                    bounds: page_rect(quad_bounds(&character.quad), bounds),
                                    font_size: character.size,
                                })
                                .collect()
                        };
                        PdfTextLine {
                            bounds: page_rect(line.bounds, bounds),
                            spans,
                        }
                    })
                    .collect(),
            }),
            _ => None,
        })
        .collect();
    let links = page
        .links()
        .map_err(mupdf_error)?
        .map(|link| {
            let (target_page, target_x, target_y) = link.dest
                .map(|destination| resolve_link_target(document, destination.loc.page_number, destination.kind))
                .unwrap_or((None, None, None));
            PdfLink {
                bounds: page_rect(link.bounds, bounds),
                uri: link.uri,
                target_page,
                target_x,
                target_y,
            }
        })
        .collect();
    Ok(PdfPageStructure {
        page_index,
        width: bounds.width(),
        height: bounds.height(),
        blocks,
        links,
    })
}

fn resolve_link_target(
    document: &Document,
    page_index: u32,
    kind: DestinationKind,
) -> (Option<u32>, Option<f32>, Option<f32>) {
    let (raw_x, raw_y) = match kind {
        DestinationKind::XYZ { left, top, .. } => (left, top),
        DestinationKind::FitH { top } | DestinationKind::FitBH { top } => (None, top),
        DestinationKind::FitV { left } | DestinationKind::FitBV { left } => (left, None),
        DestinationKind::FitR { left, top, .. } => (Some(left), Some(top)),
        DestinationKind::Fit | DestinationKind::FitB => (None, None),
    };
    let Ok(page) = document.load_page(page_index as i32) else {
        return (Some(page_index + 1), None, None);
    };
    let Ok(bounds) = page.bounds() else {
        return (Some(page_index + 1), None, None);
    };
    let x = raw_x.map(|value| ((value - bounds.x0) / bounds.width()).clamp(0.0, 1.0));
    let y = raw_y.map(|value| ((value - bounds.y0) / bounds.height()).clamp(0.0, 1.0));
    (Some(page_index + 1), x, y)
}

fn render_page(
    document: &Document,
    page_index: u32,
    scale: f32,
    authenticated: bool,
) -> AppResult<RenderedPdfPage> {
    validate_page(document, page_index, authenticated)?;
    let page = document.load_page(page_index as i32).map_err(mupdf_error)?;
    let pixmap = page
        .to_pixmap(
            &Matrix::new_scale(scale, scale),
            &Colorspace::device_rgb(),
            false,
            true,
        )
        .map_err(mupdf_error)?;
    let mut png = Vec::new();
    pixmap
        .write_to(&mut png, ImageFormat::PNG)
        .map_err(mupdf_error)?;
    Ok(RenderedPdfPage { png })
}

fn validate_page(document: &Document, page_index: u32, authenticated: bool) -> AppResult<()> {
    if document.needs_password().map_err(mupdf_error)? && !authenticated {
        return Err(AppError::PdfPasswordRequired);
    }
    if page_index >= document.page_count().map_err(mupdf_error)?.max(0) as u32 {
        return Err(AppError::PageOutOfRange);
    }
    Ok(())
}

fn page_rect(value: Rect, page_bounds: Rect) -> PdfRect {
    PdfRect {
        left: value.x0 - page_bounds.x0,
        top: value.y0 - page_bounds.y0,
        right: value.x1 - page_bounds.x0,
        bottom: value.y1 - page_bounds.y0,
    }
}

fn quad_bounds(value: &Quad) -> Rect {
    let xs = [value.ul.x, value.ur.x, value.ll.x, value.lr.x];
    let ys = [value.ul.y, value.ur.y, value.ll.y, value.lr.y];
    Rect::new(
        xs.into_iter().fold(f32::INFINITY, f32::min),
        ys.into_iter().fold(f32::INFINITY, f32::min),
        xs.into_iter().fold(f32::NEG_INFINITY, f32::max),
        ys.into_iter().fold(f32::NEG_INFINITY, f32::max),
    )
}
fn mupdf_error(error: mupdf::Error) -> AppError {
    AppError::MuPdf(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::application::ports::PdfEngine;

    use super::MupdfPdfEngine;

    fn fixture(name: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("native/mupdf-rs/tests/files")
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn opens_extracts_and_renders_a_pdf_in_its_session_worker() {
        let engine = MupdfPdfEngine::default();
        let document = engine.open(&fixture("dummy.pdf"), None).unwrap();
        assert!(document.page_count > 0);

        let page = engine.page_structure(&document.session_id, 0).unwrap();
        assert!(page.width > 0.0 && page.height > 0.0);
        let rendered = engine.render_page(&document.session_id, 0, 1.0).unwrap();
        assert!(rendered.png.starts_with(b"\x89PNG\r\n\x1a\n"));
        engine.close(&document.session_id).unwrap();
    }

    #[test]
    fn keeps_an_encrypted_document_open_until_password_authentication() {
        let engine = MupdfPdfEngine::default();
        let locked = engine.open(&fixture("dummy-encrypted.pdf"), None).unwrap();
        assert!(locked.needs_password);
        let opened = engine.authenticate(&locked.session_id, "123456").unwrap();
        assert!(!opened.needs_password);
        assert!(opened.page_count > 0);
        engine.close(&opened.session_id).unwrap();
    }
}
