use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crate::{
    application::ports::{
        DocumentRepository, MarkdownExporter, MarkdownParser, MarkdownSessionRepository,
    },
    domain::markdown_ast::{MarkdownDocumentModel, MarkdownEdit},
    error::{AppError, AppResult},
};

pub(crate) struct MarkdownDocumentService {
    documents: Arc<dyn DocumentRepository>,
    parser: Arc<dyn MarkdownParser>,
    exporter: Arc<dyn MarkdownExporter>,
    sessions: Arc<dyn MarkdownSessionRepository>,
    next_session_id: AtomicU64,
}

impl MarkdownDocumentService {
    pub(crate) fn create_untitled(&self) -> AppResult<MarkdownDocumentModel> {
        let existing_titles = self.sessions.all_titles();
        let sequence = self.next_available_sequence(&existing_titles);
        let parsed = self.parser.parse("")?;
        let session_id = format!("markdown-{sequence}");
        let document = MarkdownDocumentModel {
            document_id: session_id.clone(),
            session_id,
            revision: 1,
            path: String::new(),
            title: format!("未命名 {sequence}"),
            source: String::new(),
            nodes: parsed.nodes,
            outline: parsed.outline,
            diagnostics: parsed.diagnostics,
        };
        self.sessions.insert(document.clone())?;
        Ok(document)
    }

    fn next_available_sequence(&self, existing_titles: &[String]) -> u64 {
        for candidate in 1_u64.. {
            let title = format!("未命名 {candidate}");
            if !existing_titles.iter().any(|t| t == &title) {
                return candidate;
            }
        }
        let fallback = self.next_session_id.fetch_add(1, Ordering::Relaxed);
        fallback.max(1)
    }

    pub(crate) fn new(
        documents: Arc<dyn DocumentRepository>,
        parser: Arc<dyn MarkdownParser>,
        exporter: Arc<dyn MarkdownExporter>,
        sessions: Arc<dyn MarkdownSessionRepository>,
    ) -> Self {
        Self {
            documents,
            parser,
            exporter,
            sessions,
            next_session_id: AtomicU64::new(1),
        }
    }

    pub(crate) fn open(&self, path: &str) -> AppResult<MarkdownDocumentModel> {
        let opened = self.documents.read_text(path)?;
        let source = opened.text;
        let parsed = self.parser.parse(&source)?;
        let document = MarkdownDocumentModel {
            document_id: format!("markdown-session-{}", self.next_session_id.load(Ordering::Relaxed)),
            session_id: format!(
                "markdown-{}",
                self.next_session_id.fetch_add(1, Ordering::Relaxed)
            ),
            revision: 1,
            path: opened.path,
            title: opened.name,
            source,
            nodes: parsed.nodes,
            outline: parsed.outline,
            diagnostics: parsed.diagnostics,
        };
        self.sessions.insert(document.clone())?;
        Ok(document)
    }

    pub(crate) fn get(&self, session_id: &str) -> AppResult<MarkdownDocumentModel> {
        self.sessions.get(session_id)
    }

    pub(crate) fn apply_edits(
        &self,
        session_id: &str,
        base_revision: u64,
        edits: &[MarkdownEdit],
    ) -> AppResult<MarkdownDocumentModel> {
        let mut document = self.sessions.get(session_id)?;
        if document.revision != base_revision {
            return Err(AppError::RevisionConflict {
                expected: document.revision,
                actual: base_revision,
            });
        }

        document.source = apply_utf16_edits(&document.source, edits)?;
        let parsed = self.parser.parse(&document.source)?;
        document.revision += 1;
        document.nodes = parsed.nodes;
        document.outline = parsed.outline;
        document.diagnostics = parsed.diagnostics;
        self.sessions.replace(document.clone())?;
        Ok(document)
    }

    pub(crate) fn save(&self, session_id: &str, revision: u64) -> AppResult<()> {
        let document = self.sessions.get(session_id)?;
        if document.revision != revision {
            return Err(AppError::RevisionConflict {
                expected: document.revision,
                actual: revision,
            });
        }
        if document.path.is_empty() {
            return Err(AppError::InvalidMarkdownPath);
        }
        self.documents
            .save_markdown(&document.path, &document.source)
    }

    pub(crate) fn rebind_path(&self, session_id: &str, path: &str) -> AppResult<()> {
        let mut document = self.sessions.get(session_id)?;
        document.path = path.to_owned();
        document.title = std::path::Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or(AppError::MissingFileName)?;
        self.sessions.replace(document)
    }

    pub(crate) fn save_as(
        &self,
        session_id: &str,
        revision: u64,
        path: &str,
    ) -> AppResult<MarkdownDocumentModel> {
        let mut document = self.sessions.get(session_id)?;
        if document.revision != revision {
            return Err(AppError::RevisionConflict {
                expected: document.revision,
                actual: revision,
            });
        }
        self.documents.save_markdown(path, &document.source)?;
        document.path = path.to_string();
        document.title = std::path::Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or(AppError::MissingFileName)?;
        self.sessions.replace(document.clone())?;
        Ok(document)
    }

    pub(crate) fn export_html(&self, session_id: &str, revision: u64, path: &str) -> AppResult<()> {
        let document = self.document_at_revision(session_id, revision)?;
        self.exporter.export_html(&document, path)
    }

    pub(crate) fn export_pdf(&self, session_id: &str, revision: u64, path: &str) -> AppResult<()> {
        let document = self.document_at_revision(session_id, revision)?;
        self.exporter.export_pdf(&document, path)
    }

    pub(crate) fn close(&self, session_id: &str) -> AppResult<()> {
        self.sessions.remove(session_id)
    }

    pub(crate) fn save_untitled_recovery(&self, content: &str) -> AppResult<()> {
        self.documents.save_untitled_recovery(content)
    }

    pub(crate) fn load_untitled_recovery(&self) -> AppResult<Option<String>> {
        self.documents.load_untitled_recovery()
    }

    pub(crate) fn clear_untitled_recovery(&self) -> AppResult<()> {
        self.documents.clear_untitled_recovery()
    }

    fn document_at_revision(
        &self,
        session_id: &str,
        revision: u64,
    ) -> AppResult<MarkdownDocumentModel> {
        let document = self.sessions.get(session_id)?;
        if document.revision != revision {
            return Err(AppError::RevisionConflict {
                expected: document.revision,
                actual: revision,
            });
        }
        Ok(document)
    }
}

fn apply_utf16_edits(source: &str, edits: &[MarkdownEdit]) -> AppResult<String> {
    let mut normalized = edits
        .iter()
        .map(|edit| {
            let start = utf16_offset_to_byte(source, edit.start_utf16)?;
            let end = utf16_offset_to_byte(source, edit.end_utf16)?;
            if start > end {
                return Err(AppError::InvalidMarkdownEdit);
            }
            Ok((start, end, edit.text.as_str()))
        })
        .collect::<AppResult<Vec<_>>>()?;

    normalized.sort_by_key(|(start, _, _)| *start);
    if normalized.windows(2).any(|pair| pair[0].1 > pair[1].0) {
        return Err(AppError::InvalidMarkdownEdit);
    }

    let mut output = source.to_string();
    for (start, end, text) in normalized.into_iter().rev() {
        output.replace_range(start..end, text);
    }
    Ok(output)
}

fn utf16_offset_to_byte(value: &str, offset: usize) -> AppResult<usize> {
    if offset == value.encode_utf16().count() {
        return Ok(value.len());
    }

    let mut utf16_index = 0;
    for (byte_index, ch) in value.char_indices() {
        if utf16_index == offset {
            return Ok(byte_index);
        }
        utf16_index += ch.len_utf16();
        if utf16_index > offset {
            return Err(AppError::InvalidMarkdownEdit);
        }
    }
    Err(AppError::InvalidMarkdownEdit)
}

#[cfg(test)]
mod tests {
    use crate::domain::markdown_ast::MarkdownEdit;

    use super::apply_utf16_edits;

    #[test]
    fn applies_javascript_utf16_ranges_without_splitting_unicode() {
        let source = "A😀B";
        let output = apply_utf16_edits(
            source,
            &[MarkdownEdit {
                start_utf16: 1,
                end_utf16: 3,
                text: "好".into(),
            }],
        )
        .unwrap();
        assert_eq!(output, "A好B");
    }

    #[test]
    fn rejects_ranges_inside_surrogate_pairs() {
        assert!(apply_utf16_edits(
            "A😀B",
            &[MarkdownEdit {
                start_utf16: 2,
                end_utf16: 3,
                text: String::new(),
            }]
        )
        .is_err());
    }
}
