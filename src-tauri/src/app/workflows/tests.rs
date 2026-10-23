use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use super::{document_lifecycle, pdf_lifecycle};
use crate::{
    app::state::AppState,
    features::{
        markdown::{InMemoryMarkdownSessions, MarkdownEdit, MarkdownSearchAdapter},
        pdf::{
            MupdfPdfEngine, PdfDocumentModel, PdfDocumentService, PdfEngine, PdfPageStructure,
            RenderedPdfPage,
        },
        search::{
            DocumentSearchService, PdfSearchEngine, SearchOptions, SearchRequest, SearchResponse,
            SearchTarget,
        },
    },
    shared::error::{AppError, AppResult},
};

struct Sandbox {
    state: AppState,
    root: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "bambook-workflows-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let state = AppState::new(root.clone()).unwrap();
        Self { state, root }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("native/mupdf-rs/tests/files")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn query(target: SearchTarget, text: &str) -> SearchRequest {
    SearchRequest {
        request_id: 7,
        target,
        query: text.into(),
        options: SearchOptions::default(),
    }
}

#[test]
fn pdf_workflow_opens_searches_and_releases_both_sessions() {
    let sandbox = Sandbox::new();
    let state = &sandbox.state;
    let document = pdf_lifecycle::open(state, &fixture("dummy.pdf"), None).unwrap();
    let target = SearchTarget::Pdf {
        session_id: document.session_id.clone(),
    };
    assert!(
        state
            .search
            .search(query(target.clone(), "Dummy"))
            .unwrap()
            .total
            > 0
    );
    assert!(state
        .pdf_documents
        .render_page(&document.session_id, 0, 1.0)
        .unwrap()
        .png
        .starts_with(b"\x89PNG"));
    pdf_lifecycle::close(state, &document.session_id).unwrap();
    assert!(matches!(
        state.pdf_documents.page_structure(&document.session_id, 0),
        Err(AppError::PdfSessionNotFound)
    ));
    assert!(matches!(
        state.search.search(query(target, "Dummy")),
        Err(AppError::PdfSessionNotFound)
    ));
}

#[test]
fn encrypted_pdf_authentication_reaches_display_and_search() {
    let sandbox = Sandbox::new();
    let state = &sandbox.state;
    let document = pdf_lifecycle::open(state, &fixture("dummy-encrypted.pdf"), None).unwrap();
    let target = SearchTarget::Pdf {
        session_id: document.session_id.clone(),
    };
    assert!(document.needs_password);
    assert!(matches!(
        state.search.search(query(target.clone(), "test")),
        Err(AppError::PdfPasswordRequired)
    ));
    let authenticated = pdf_lifecycle::authenticate(state, &document.session_id, "123456").unwrap();
    assert!(!authenticated.needs_password);
    state
        .pdf_documents
        .page_structure(&document.session_id, 0)
        .unwrap();
    state.search.search(query(target, "test")).unwrap();
    pdf_lifecycle::close(state, &document.session_id).unwrap();
}

#[test]
fn markdown_edits_are_searchable_then_saved_and_reopened_from_the_library() {
    let sandbox = Sandbox::new();
    let state = &sandbox.state;
    let source = sandbox.root.join("notes.md");
    fs::write(&source, "# Before\n").unwrap();
    let document = document_lifecycle::open_markdown(state, source.to_str().unwrap()).unwrap();
    let edited = state
        .markdown_documents
        .apply_edits(
            &document.session_id,
            document.revision,
            &[MarkdownEdit {
                start_utf16: 0,
                end_utf16: 9,
                text: "# After\n中文 😀\n".into(),
            }],
        )
        .unwrap();
    let target = SearchTarget::Markdown {
        session_id: document.session_id.clone(),
        revision: edited.revision,
    };
    assert_eq!(
        state
            .search
            .search(query(target.clone(), "中文"))
            .unwrap()
            .total,
        1
    );
    state.search.cancel(&target, 7).unwrap();
    document_lifecycle::save_markdown(state, &document.session_id, edited.revision).unwrap();
    assert!(fs::read_to_string(&source).unwrap().contains("中文 😀"));
    let stored = state
        .library
        .load_document_library()
        .unwrap()
        .pop()
        .unwrap();
    document_lifecycle::close_markdown(state, &document.session_id).unwrap();
    assert!(matches!(
        state.search.search(query(target, "中文")),
        Err(AppError::MarkdownSessionNotFound)
    ));
    let reopened = document_lifecycle::open_markdown(state, &stored.path).unwrap();
    assert!(reopened.source.contains("中文 😀"));
    document_lifecycle::close_markdown(state, &reopened.session_id).unwrap();
}

struct FailingSearch {
    reject_registration: bool,
}

impl PdfSearchEngine for FailingSearch {
    fn register(&self, _: &str, _: &str, _: Option<&str>) -> AppResult<()> {
        if self.reject_registration {
            Err(AppError::MuPdf("registration failed".into()))
        } else {
            Ok(())
        }
    }
    fn authenticate(&self, _: &str, _: &str) -> AppResult<()> {
        unreachable!()
    }
    fn search(&self, _: &str, _: u64, _: &str, _: &SearchOptions) -> AppResult<SearchResponse> {
        unreachable!()
    }
    fn cancel(&self, _: &str, _: u64) -> AppResult<()> {
        unreachable!()
    }
    fn close(&self, _: &str) -> AppResult<()> {
        Err(AppError::MuPdf("close failed".into()))
    }
}

#[derive(Default)]
struct RecordingEngine {
    inner: MupdfPdfEngine,
    last_session: Mutex<Option<String>>,
}

impl PdfEngine for RecordingEngine {
    fn open(&self, path: &str, password: Option<&str>) -> AppResult<PdfDocumentModel> {
        let document = self.inner.open(path, password)?;
        *self.last_session.lock().unwrap() = Some(document.session_id.clone());
        Ok(document)
    }
    fn authenticate(&self, id: &str, password: &str) -> AppResult<PdfDocumentModel> {
        self.inner.authenticate(id, password)
    }
    fn page_structure(&self, id: &str, page: u32) -> AppResult<PdfPageStructure> {
        self.inner.page_structure(id, page)
    }
    fn render_page(&self, id: &str, page: u32, scale: f32) -> AppResult<RenderedPdfPage> {
        self.inner.render_page(id, page, scale)
    }
    fn close(&self, id: &str) -> AppResult<()> {
        self.inner.close(id)
    }
}

#[test]
fn failed_search_registration_releases_the_opened_pdf_worker() {
    let mut sandbox = Sandbox::new();
    let engine = Arc::new(RecordingEngine::default());
    sandbox.state.pdf_documents = Arc::new(PdfDocumentService::new(
        engine.clone(),
        sandbox.state.library.clone(),
    ));
    sandbox.state.search = Arc::new(DocumentSearchService::new(
        Arc::new(FailingSearch {
            reject_registration: true,
        }),
        Arc::new(MarkdownSearchAdapter::new(Arc::new(
            InMemoryMarkdownSessions::default(),
        ))),
    ));
    assert!(pdf_lifecycle::open(&sandbox.state, &fixture("dummy.pdf"), None).is_err());
    let id = engine.last_session.lock().unwrap().clone().unwrap();
    assert!(matches!(
        engine.page_structure(&id, 0),
        Err(AppError::PdfSessionNotFound)
    ));
}

#[test]
fn failed_search_close_still_releases_the_display_worker() {
    let mut sandbox = Sandbox::new();
    let document = pdf_lifecycle::open(&sandbox.state, &fixture("dummy.pdf"), None).unwrap();
    // Release the original search worker before replacing the provider for fault injection.
    sandbox
        .state
        .search
        .close_pdf(&document.session_id)
        .unwrap();
    sandbox.state.search = Arc::new(DocumentSearchService::new(
        Arc::new(FailingSearch {
            reject_registration: false,
        }),
        Arc::new(MarkdownSearchAdapter::new(Arc::new(
            InMemoryMarkdownSessions::default(),
        ))),
    ));
    assert!(pdf_lifecycle::close(&sandbox.state, &document.session_id).is_err());
    assert!(matches!(
        sandbox
            .state
            .pdf_documents
            .page_structure(&document.session_id, 0),
        Err(AppError::PdfSessionNotFound)
    ));
}
