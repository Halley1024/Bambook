use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use mupdf::Document;

use crate::{
    application::ports::PdfSearchEngine,
    domain::search::{SearchOptions, SearchResponse},
    error::{AppError, AppResult},
};

use super::search_index::PdfPageSearchIndex;

struct SearchSession {
    sender: Sender<SearchRequest>,
    generation: Arc<AtomicU64>,
}

enum SearchRequest {
    Authenticate(String, Sender<AppResult<()>>),
    Search {
        request_id: u64,
        query: String,
        options: SearchOptions,
        reply: Sender<AppResult<SearchResponse>>,
    },
    Close,
}

#[derive(Default)]
pub(crate) struct MupdfPdfSearchEngine {
    sessions: Mutex<HashMap<String, SearchSession>>,
}

impl PdfSearchEngine for MupdfPdfSearchEngine {
    fn register(&self, session_id: &str, path: &str, password: Option<&str>) -> AppResult<()> {
        let (sender, receiver) = mpsc::channel();
        let (ready_sender, ready_receiver) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let worker_generation = Arc::clone(&generation);
        let worker_path = path.to_owned();
        let worker_password = password.map(str::to_owned);
        thread::Builder::new()
            .name(format!("mupdf-search-{session_id}"))
            .spawn(move || {
                run_worker(
                    worker_path,
                    worker_password,
                    receiver,
                    ready_sender,
                    worker_generation,
                )
            })
            .map_err(|error| AppError::MuPdf(error.to_string()))?;
        ready_receiver
            .recv()
            .map_err(|_| AppError::StateUnavailable)??;
        self.sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .insert(session_id.to_owned(), SearchSession { sender, generation });
        Ok(())
    }

    fn authenticate(&self, session_id: &str, password: &str) -> AppResult<()> {
        let sender = self.sender(session_id)?;
        let (reply, receiver) = mpsc::channel();
        sender
            .send(SearchRequest::Authenticate(password.to_owned(), reply))
            .map_err(|_| AppError::StateUnavailable)?;
        receiver.recv().map_err(|_| AppError::StateUnavailable)?
    }

    fn search(
        &self,
        session_id: &str,
        request_id: u64,
        query: &str,
        options: &SearchOptions,
    ) -> AppResult<SearchResponse> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        let session = sessions
            .get(session_id)
            .ok_or(AppError::PdfSessionNotFound)?;
        session.generation.store(request_id, Ordering::Release);
        let sender = session.sender.clone();
        drop(sessions);
        let (reply, receiver) = mpsc::channel();
        sender
            .send(SearchRequest::Search {
                request_id,
                query: query.to_owned(),
                options: options.clone(),
                reply,
            })
            .map_err(|_| AppError::StateUnavailable)?;
        receiver.recv().map_err(|_| AppError::StateUnavailable)?
    }

    fn cancel(&self, session_id: &str, request_id: u64) -> AppResult<()> {
        let sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        let session = sessions
            .get(session_id)
            .ok_or(AppError::PdfSessionNotFound)?;
        let _ =
            session
                .generation
                .compare_exchange(request_id, 0, Ordering::AcqRel, Ordering::Acquire);
        Ok(())
    }

    fn close(&self, session_id: &str) -> AppResult<()> {
        let session = self
            .sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .remove(session_id)
            .ok_or(AppError::PdfSessionNotFound)?;
        session.generation.store(0, Ordering::Release);
        session
            .sender
            .send(SearchRequest::Close)
            .map_err(|_| AppError::StateUnavailable)
    }
}

impl MupdfPdfSearchEngine {
    fn sender(&self, session_id: &str) -> AppResult<Sender<SearchRequest>> {
        self.sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .get(session_id)
            .map(|session| session.sender.clone())
            .ok_or(AppError::PdfSessionNotFound)
    }
}

fn run_worker(
    path: String,
    password: Option<String>,
    requests: Receiver<SearchRequest>,
    ready: Sender<AppResult<()>>,
    generation: Arc<AtomicU64>,
) {
    let mut document = match Document::open(&path).map_err(mupdf_error) {
        Ok(document) => document,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    let mut authenticated = !document.needs_password().unwrap_or(false);
    if let Some(password) = password {
        authenticated = document.authenticate(&password).unwrap_or(false);
    }
    if ready.send(Ok(())).is_err() {
        return;
    }
    let mut pages = HashMap::<u32, PdfPageSearchIndex>::new();
    while let Ok(request) = requests.recv() {
        match request {
            SearchRequest::Authenticate(password, reply) => {
                let result = document
                    .authenticate(&password)
                    .map_err(mupdf_error)
                    .and_then(|accepted| {
                        if accepted {
                            authenticated = true;
                            pages.clear();
                            Ok(())
                        } else {
                            Err(AppError::PdfInvalidPassword)
                        }
                    });
                let _ = reply.send(result);
            }
            SearchRequest::Search {
                request_id,
                query,
                options,
                reply,
            } => {
                let result = search_document(
                    &document,
                    authenticated,
                    &mut pages,
                    &generation,
                    request_id,
                    &query,
                    &options,
                );
                let _ = reply.send(result);
            }
            SearchRequest::Close => break,
        }
    }
}

fn search_document(
    document: &Document,
    authenticated: bool,
    cache: &mut HashMap<u32, PdfPageSearchIndex>,
    generation: &AtomicU64,
    request_id: u64,
    query: &str,
    options: &SearchOptions,
) -> AppResult<SearchResponse> {
    if !authenticated {
        return Err(AppError::PdfPasswordRequired);
    }
    let page_count = document.page_count().map_err(mupdf_error)?.max(0) as u32;
    let mut matches = Vec::new();
    let mut total = 0;
    for page_index in 0..page_count {
        if generation.load(Ordering::Acquire) != request_id {
            return Ok(SearchResponse {
                request_id,
                total: 0,
                truncated: false,
                cancelled: true,
                matches: Vec::new(),
            });
        }
        if !cache.contains_key(&page_index) {
            cache.insert(page_index, PdfPageSearchIndex::build(document, page_index)?);
        }
        let page_matches = cache
            .get(&page_index)
            .expect("page index inserted")
            .search(request_id, query, options);
        total += page_matches.len();
        let remaining = options.result_limit.saturating_sub(matches.len());
        matches.extend(page_matches.into_iter().take(remaining));
    }
    Ok(SearchResponse {
        request_id,
        total,
        truncated: total > matches.len(),
        cancelled: false,
        matches,
    })
}

fn mupdf_error(error: mupdf::Error) -> AppError {
    AppError::MuPdf(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::application::ports::PdfSearchEngine;

    use super::*;

    fn fixture(name: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("native/mupdf-rs/tests/files")
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn authenticates_and_releases_an_encrypted_search_worker() {
        let engine = MupdfPdfSearchEngine::default();
        engine
            .register("search-locked", &fixture("dummy-encrypted.pdf"), None)
            .unwrap();
        let locked = engine.search("search-locked", 1, "test", &SearchOptions::default());
        assert!(matches!(locked, Err(AppError::PdfPasswordRequired)));

        engine.authenticate("search-locked", "123456").unwrap();
        let response = engine
            .search("search-locked", 2, "test", &SearchOptions::default())
            .unwrap();
        assert!(!response.cancelled);
        engine.close("search-locked").unwrap();
        assert!(matches!(
            engine.search("search-locked", 3, "test", &SearchOptions::default()),
            Err(AppError::PdfSessionNotFound)
        ));
    }
}
