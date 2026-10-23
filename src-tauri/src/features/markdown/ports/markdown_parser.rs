use crate::features::markdown::ParsedMarkdown;
use crate::shared::error::AppResult;

pub(crate) trait MarkdownParser: Send + Sync {
    fn parse(&self, source: &str) -> AppResult<ParsedMarkdown>;
}
