use crate::{
    domain::search::{SearchOptions, SearchResponse},
    error::AppResult,
};

pub(crate) trait PdfSearchEngine: Send + Sync {
    fn register(&self, session_id: &str, path: &str, password: Option<&str>) -> AppResult<()>;
    fn authenticate(&self, session_id: &str, password: &str) -> AppResult<()>;
    fn search(
        &self,
        session_id: &str,
        request_id: u64,
        query: &str,
        options: &SearchOptions,
    ) -> AppResult<SearchResponse>;
    fn cancel(&self, session_id: &str, request_id: u64) -> AppResult<()>;
    fn close(&self, session_id: &str) -> AppResult<()>;
}
