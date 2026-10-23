use crate::features::search::{SearchRequest, SearchResponse};
use crate::shared::error::AppResult;

pub(crate) trait MarkdownSearchProvider: Send + Sync {
    fn search(&self, request: &SearchRequest) -> AppResult<SearchResponse>;
    fn cancel(&self, session_id: &str, request_id: u64) -> AppResult<()>;
    fn close(&self, session_id: &str);
}
