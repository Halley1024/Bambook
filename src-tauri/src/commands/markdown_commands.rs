use std::sync::Arc;

use tauri::State;

use crate::{
    commands::dto::{MarkdownDocumentDto, MarkdownEditDto},
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn create_untitled_markdown_document(
    state: State<'_, AppState>,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(&state.markdown_documents);
    tauri::async_runtime::spawn_blocking(move || service.create_untitled().map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn open_markdown_document(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(&state.markdown_documents);
    let storage = Arc::clone(&state.storage);
    let settings = state.settings.load()?;
    tauri::async_runtime::spawn_blocking(move || {
        let managed = storage.prepare_markdown(&path, &settings)?;
        let mut document = service.open(&managed.storage_path)?;
        document.document_id = managed.document_id;
        document.path = managed.storage_path;
        document.title = managed.display_name;
        Ok(document.into())
    })
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn get_markdown_structure(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(&state.markdown_documents);
    tauri::async_runtime::spawn_blocking(move || service.get(&session_id).map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn apply_markdown_edits(
    state: State<'_, AppState>,
    session_id: String,
    base_revision: u64,
    edits: Vec<MarkdownEditDto>,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(&state.markdown_documents);
    let edits = edits.into_iter().map(Into::into).collect::<Vec<_>>();
    tauri::async_runtime::spawn_blocking(move || {
        service
            .apply_edits(&session_id, base_revision, &edits)
            .map(Into::into)
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_markdown_document(
    state: State<'_, AppState>,
    session_id: String,
    revision: u64,
) -> AppResult<()> {
    let service = Arc::clone(&state.markdown_documents);
    let storage = Arc::clone(&state.storage);
    let settings = state.settings.load()?;
    tauri::async_runtime::spawn_blocking(move || {
        let current = service.get(&session_id)?;
        if let Some(entry) = storage.load_document_library()?.into_iter()
            .find(|entry| entry.id == current.document_id)
        {
            let recorded_path_available = std::path::Path::new(&entry.path).is_file();
            let source_available = std::path::Path::new(&entry.source_path).is_file();
            let save_path = if recorded_path_available {
                Some(entry.path.as_str())
            } else if source_available {
                Some(entry.source_path.as_str())
            } else {
                None
            };
            if let Some(save_path) = save_path.filter(|path| *path != current.path) {
                service.rebind_path(&session_id, save_path)?;
            }
        }
        service.save(&session_id, revision)?;
        let document = service.get(&session_id)?;
        if !document.path.is_empty() { storage.persist_markdown(&document.path, &settings)?; }
        Ok(())
    })
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn save_markdown_document_as(
    state: State<'_, AppState>,
    session_id: String,
    revision: u64,
    path: String,
) -> AppResult<MarkdownDocumentDto> {
    let service = Arc::clone(&state.markdown_documents);
    let storage = Arc::clone(&state.storage);
    let settings = state.settings.load()?;
    tauri::async_runtime::spawn_blocking(move || {
        let document = service.save_as(&session_id, revision, &path)?;
        storage.persist_markdown(&document.path, &settings)?;
        Ok(document.into())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn export_markdown_html(
    state: State<'_, AppState>,
    session_id: String,
    revision: u64,
    path: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.markdown_documents);
    tauri::async_runtime::spawn_blocking(move || service.export_html(&session_id, revision, &path))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn export_markdown_pdf(
    state: State<'_, AppState>,
    session_id: String,
    revision: u64,
    path: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.markdown_documents);
    tauri::async_runtime::spawn_blocking(move || service.export_pdf(&session_id, revision, &path))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn close_markdown_document(
    state: State<'_, AppState>,
    session_id: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.markdown_documents);
    let search = Arc::clone(&state.search);
    let search_session_id = session_id.clone();
    tauri::async_runtime::spawn_blocking(move || service.close(&session_id))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))??;
    search.close_markdown(&search_session_id);
    Ok(())
}

#[tauri::command]
pub(crate) async fn save_untitled_markdown_recovery(
    state: State<'_, AppState>,
    content: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.markdown_documents);
    tauri::async_runtime::spawn_blocking(move || service.save_untitled_recovery(&content))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn load_untitled_markdown_recovery(
    state: State<'_, AppState>,
) -> AppResult<Option<String>> {
    let service = Arc::clone(&state.markdown_documents);
    tauri::async_runtime::spawn_blocking(move || service.load_untitled_recovery())
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn clear_untitled_markdown_recovery(
    state: State<'_, AppState>,
) -> AppResult<()> {
    let service = Arc::clone(&state.markdown_documents);
    tauri::async_runtime::spawn_blocking(move || service.clear_untitled_recovery())
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
