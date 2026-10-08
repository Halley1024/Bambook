use std::{collections::HashMap, sync::Mutex};

use crate::{
    application::ports::MarkdownSessionRepository,
    domain::markdown_ast::MarkdownDocumentModel,
    error::{AppError, AppResult},
};

#[derive(Default)]
pub(crate) struct InMemoryMarkdownSessions {
    sessions: Mutex<HashMap<String, MarkdownDocumentModel>>,
}

impl MarkdownSessionRepository for InMemoryMarkdownSessions {
    fn insert(&self, document: MarkdownDocumentModel) -> AppResult<()> {
        self.sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .insert(document.session_id.clone(), document);
        Ok(())
    }

    fn get(&self, session_id: &str) -> AppResult<MarkdownDocumentModel> {
        self.sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .get(session_id)
            .cloned()
            .ok_or(AppError::MarkdownSessionNotFound)
    }

    fn replace(&self, document: MarkdownDocumentModel) -> AppResult<()> {
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        if !sessions.contains_key(&document.session_id) {
            return Err(AppError::MarkdownSessionNotFound);
        }
        sessions.insert(document.session_id.clone(), document);
        Ok(())
    }

    fn remove(&self, session_id: &str) -> AppResult<()> {
        self.sessions
            .lock()
            .map_err(|_| AppError::StateUnavailable)?
            .remove(session_id)
            .map(|_| ())
            .ok_or(AppError::MarkdownSessionNotFound)
    }

    fn all_titles(&self) -> Vec<String> {
        self.sessions
            .lock()
            .map(|guard| guard.values().map(|doc| doc.title.clone()).collect())
            .unwrap_or_default()
    }
}
