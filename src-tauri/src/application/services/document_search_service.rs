use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    application::ports::{MarkdownSessionRepository, PdfSearchEngine},
    domain::search::{
        MarkdownSearchLocator, SearchLocator, SearchMatch, SearchRequest, SearchResponse,
        SearchTarget,
    },
    error::{AppError, AppResult},
    infrastructure::search::{find_matches, normalize_symbols, SourceSymbol},
};

pub(crate) struct DocumentSearchService {
    pdf: Arc<dyn PdfSearchEngine>,
    markdown: Arc<dyn MarkdownSessionRepository>,
    markdown_generations: Mutex<HashMap<String, Arc<AtomicU64>>>,
}

impl DocumentSearchService {
    pub(crate) fn new(
        pdf: Arc<dyn PdfSearchEngine>,
        markdown: Arc<dyn MarkdownSessionRepository>,
    ) -> Self {
        Self {
            pdf,
            markdown,
            markdown_generations: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn register_pdf(
        &self,
        session_id: &str,
        path: &str,
        password: Option<&str>,
    ) -> AppResult<()> {
        self.pdf.register(session_id, path, password)
    }

    pub(crate) fn authenticate_pdf(&self, session_id: &str, password: &str) -> AppResult<()> {
        self.pdf.authenticate(session_id, password)
    }

    pub(crate) fn close_pdf(&self, session_id: &str) -> AppResult<()> {
        self.pdf.close(session_id)
    }

    pub(crate) fn search(&self, request: SearchRequest) -> AppResult<SearchResponse> {
        match &request.target {
            SearchTarget::Pdf { session_id } => self.pdf.search(
                session_id,
                request.request_id,
                &request.query,
                &request.options,
            ),
            SearchTarget::Markdown {
                session_id,
                revision,
            } => {
                let generation = self.markdown_generation(session_id)?;
                generation.store(request.request_id, Ordering::Release);
                let document = self.markdown.get(session_id)?;
                if document.revision != *revision {
                    return Err(AppError::RevisionConflict {
                        expected: document.revision,
                        actual: *revision,
                    });
                }
                search_markdown(&document.source, document.revision, &request, &generation)
            }
        }
    }

    pub(crate) fn cancel(&self, target: &SearchTarget, request_id: u64) -> AppResult<()> {
        match target {
            SearchTarget::Pdf { session_id } => self.pdf.cancel(session_id, request_id),
            SearchTarget::Markdown { session_id, .. } => {
                let generation = self.markdown_generation(session_id)?;
                let _ =
                    generation.compare_exchange(request_id, 0, Ordering::AcqRel, Ordering::Acquire);
                Ok(())
            }
        }
    }

    pub(crate) fn close_markdown(&self, session_id: &str) {
        if let Ok(mut generations) = self.markdown_generations.lock() {
            if let Some(generation) = generations.remove(session_id) {
                generation.store(0, Ordering::Release);
            }
        }
    }

    fn markdown_generation(&self, session_id: &str) -> AppResult<Arc<AtomicU64>> {
        let mut generations = self
            .markdown_generations
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        Ok(Arc::clone(
            generations
                .entry(session_id.to_owned())
                .or_insert_with(|| Arc::new(AtomicU64::new(0))),
        ))
    }
}

fn search_markdown(
    source: &str,
    revision: u64,
    request: &SearchRequest,
    generation: &AtomicU64,
) -> AppResult<SearchResponse> {
    let symbols = source_symbols(source);
    let normalized = normalize_symbols(&symbols, request.options.case_sensitive, false);
    let query = normalize_symbols(
        &source_symbols(&request.query),
        request.options.case_sensitive,
        false,
    )
    .value;
    let ranges = find_matches(&normalized, query.trim(), request.options.whole_word);
    if generation.load(Ordering::Acquire) != request.request_id {
        return Ok(SearchResponse {
            request_id: request.request_id,
            total: 0,
            truncated: false,
            cancelled: true,
            matches: Vec::new(),
        });
    }
    let total = ranges.len();
    let matches = ranges
        .into_iter()
        .take(request.options.result_limit)
        .enumerate()
        .filter_map(|(ordinal, range)| {
            let units = &normalized.units[range.first_unit..=range.last_unit];
            let start = units.iter().map(|unit| unit.source_start).min()?;
            let end = units.iter().map(|unit| unit.source_end).max()?;
            let start_utf16 = source[..start].encode_utf16().count();
            let end_utf16 = source[..end].encode_utf16().count();
            let line = source[..start]
                .chars()
                .filter(|character| *character == '\n')
                .count()
                + 1;
            let line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);
            let column = source[line_start..start].chars().count() + 1;
            let (before_text, matched_text, after_text) = context(source, start, end, 40);
            Some(SearchMatch {
                id: format!("markdown-{}-{ordinal}", request.request_id),
                matched_text,
                before_text,
                after_text,
                location_label: format!("第 {line} 行"),
                locator: SearchLocator::Markdown(MarkdownSearchLocator {
                    revision,
                    start_utf16,
                    end_utf16,
                    line,
                    column,
                }),
            })
        })
        .collect::<Vec<_>>();
    Ok(SearchResponse {
        request_id: request.request_id,
        total,
        truncated: total > matches.len(),
        cancelled: false,
        matches,
    })
}

fn source_symbols(source: &str) -> Vec<SourceSymbol> {
    let mut line = 1;
    source
        .char_indices()
        .map(|(start, character)| {
            let symbol = SourceSymbol {
                text: character.to_string(),
                source_start: start,
                source_end: start + character.len_utf8(),
                line,
                rect: None,
            };
            if character == '\n' {
                line += 1;
            }
            symbol
        })
        .collect()
}

fn context(source: &str, start: usize, end: usize, radius: usize) -> (String, String, String) {
    let before_start = source[..start]
        .char_indices()
        .rev()
        .nth(radius)
        .map_or(0, |(index, _)| index);
    let after_end = source[end..]
        .char_indices()
        .nth(radius)
        .map_or(source.len(), |(index, _)| end + index);
    (
        source[before_start..start].replace('\n', " "),
        source[start..end].replace('\n', " "),
        source[end..after_end].replace('\n', " "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::search::{SearchOptions, SearchTarget};

    #[test]
    fn markdown_results_use_javascript_utf16_offsets() {
        let request = SearchRequest {
            request_id: 1,
            target: SearchTarget::Markdown {
                session_id: "m".into(),
                revision: 2,
            },
            query: "中文".into(),
            options: SearchOptions::default(),
        };
        let generation = AtomicU64::new(1);
        let result = search_markdown("😀 中文 test", 2, &request, &generation).unwrap();
        let SearchLocator::Markdown(locator) = &result.matches[0].locator else {
            panic!()
        };
        assert_eq!(locator.start_utf16, 3);
        assert_eq!(locator.end_utf16, 5);
    }
}
