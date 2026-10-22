use crate::app::state::AppState;
use crate::features::markdown::MarkdownDocumentModel;
use crate::shared::error::AppResult;

pub(crate) fn open_markdown(state: &AppState, path: &str) -> AppResult<MarkdownDocumentModel> {
    let settings = state.settings.load()?;
    let managed = state.library.prepare_markdown(path, &settings)?;
    let mut document = state.markdown_documents.open(&managed.storage_path)?;
    document.document_id = managed.document_id;
    document.path = managed.storage_path;
    document.title = managed.display_name;
    Ok(document)
}
pub(crate) fn save_markdown(state: &AppState, session_id: &str, revision: u64) -> AppResult<()> {
    let settings = state.settings.load()?;
    let service = &state.markdown_documents;
    let current = service.get(session_id)?;
    if let Some(entry) = state
        .library
        .load_document_library()?
        .into_iter()
        .find(|entry| entry.id == current.document_id)
    {
        let save_path = if std::path::Path::new(&entry.path).is_file() {
            Some(entry.path.as_str())
        } else if std::path::Path::new(&entry.source_path).is_file() {
            Some(entry.source_path.as_str())
        } else {
            None
        };
        if let Some(path) = save_path.filter(|path| *path != current.path) {
            service.rebind_path(session_id, path)?;
        }
    }
    service.save(session_id, revision)?;
    let document = service.get(session_id)?;
    if !document.path.is_empty() {
        state.library.persist_markdown(&document.path, &settings)?;
    }
    Ok(())
}
pub(crate) fn save_markdown_as(
    state: &AppState,
    session_id: &str,
    revision: u64,
    path: &str,
) -> AppResult<MarkdownDocumentModel> {
    let settings = state.settings.load()?;
    let document = state
        .markdown_documents
        .save_as(session_id, revision, path)?;
    state.library.persist_markdown(&document.path, &settings)?;
    Ok(document)
}
pub(crate) fn close_markdown(state: &AppState, session_id: &str) -> AppResult<()> {
    state.markdown_documents.close(session_id)?;
    state.search.close_markdown(session_id);
    Ok(())
}
pub(crate) fn reconcile_storage(state: &AppState, document_id: &str) -> AppResult<()> {
    state
        .library
        .reconcile_document_storage(document_id, &state.settings.load()?)
}
