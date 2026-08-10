use std::sync::Arc;

use tauri::State;

use crate::{
    commands::dto::{DocumentKindDto, MarkdownFileEntryDto},
    error::{AppError, AppResult},
    state::AppState,
};

#[tauri::command]
pub(crate) async fn detect_document_kind(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<DocumentKindDto> {
    let service = Arc::clone(&state.documents);
    tauri::async_runtime::spawn_blocking(move || service.detect_kind(&path).map(Into::into))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn create_markdown_document(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<()> {
    let service = Arc::clone(&state.documents);
    tauri::async_runtime::spawn_blocking(move || service.create_markdown(&path))
        .await
        .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn rename_markdown_document(path: String, new_path: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        if !new_path.ends_with(".md") {
            return Err(AppError::InvalidMarkdownPath);
        }
        std::fs::rename(&path, &new_path)?;
        Ok(())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn rename_pdf_document(path: String, new_path: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::rename(&path, &new_path)?;
        Ok(())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn reveal_in_file_manager(path: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        std::process::Command::new("explorer.exe")
            .arg(format!("/select,{path}"))
            .spawn()?;
        Ok(())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn open_directory_in_file_manager(path: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        std::process::Command::new("explorer.exe")
            .arg(path)
            .spawn()?;
        Ok(())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn open_external_url(url: String) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let target = url.trim();
        if !is_allowed_external_url(target) {
            return Err(AppError::UnsafeExternalLink);
        }
        std::process::Command::new("rundll32.exe")
            .arg("url.dll,FileProtocolHandler")
            .arg(target)
            .spawn()?;
        Ok(())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

fn is_allowed_external_url(value: &str) -> bool {
    if value.contains('\0') { return false; }
    let lower = value.trim().to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:")
}

#[cfg(test)]
mod external_link_tests {
    use super::is_allowed_external_url;

    #[test]
    fn external_link_protocol_allowlist_rejects_executable_and_local_targets() {
        assert!(is_allowed_external_url("https://example.com/path"));
        assert!(is_allowed_external_url("HTTP://example.com"));
        assert!(is_allowed_external_url("mailto:reader@example.com"));
        assert!(!is_allowed_external_url("javascript:alert(1)"));
        assert!(!is_allowed_external_url("data:text/html,test"));
        assert!(!is_allowed_external_url("file:///C:/secret.txt"));
        assert!(!is_allowed_external_url("C:\\secret.txt"));
    }
}

#[tauri::command]
pub(crate) async fn list_markdown_directory(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<Vec<MarkdownFileEntryDto>> {
    let service = Arc::clone(&state.documents);
    tauri::async_runtime::spawn_blocking(move || {
        service
            .list_markdown_directory(&path)
            .map(|entries| entries.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}

#[tauri::command]
pub(crate) async fn list_supported_directory(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<Vec<MarkdownFileEntryDto>> {
    let service = Arc::clone(&state.documents);
    tauri::async_runtime::spawn_blocking(move || {
        service
            .list_supported_directory(&path)
            .map(|entries| entries.into_iter().map(Into::into).collect())
    })
    .await
    .map_err(|error| AppError::BackgroundTask(error.to_string()))?
}
