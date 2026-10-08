use crate::{domain::markdown_ast::ParsedMarkdown, error::AppResult};

pub(crate) trait MarkdownParser: Send + Sync {
    fn parse(&self, source: &str) -> AppResult<ParsedMarkdown>;
}
