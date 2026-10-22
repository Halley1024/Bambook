use crate::features::documents::is_markdown_path;
use crate::features::documents::OpenedDocument;
use crate::features::markdown::MarkdownFileRepository;
use crate::platform::paths::AppPaths;
use crate::shared::error::{AppError, AppResult};
use std::{fs, path::Path};

pub(crate) struct FileMarkdownRepository {
    paths: AppPaths,
}
impl FileMarkdownRepository {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
    fn file_name(path: &str) -> AppResult<String> {
        Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or(AppError::MissingFileName)
    }
}
impl MarkdownFileRepository for FileMarkdownRepository {
    fn create_markdown(&self, path: &str) -> AppResult<()> {
        self.save_markdown(path, "")
    }

    fn read_text(&self, path: &str) -> AppResult<OpenedDocument> {
        Ok(OpenedDocument {
            name: Self::file_name(path)?,
            path: path.to_string(),
            text: fs::read_to_string(path)?,
        })
    }

    fn save_markdown(&self, path: &str, text: &str) -> AppResult<()> {
        let path = Path::new(path);
        if !is_markdown_path(path) {
            return Err(AppError::InvalidMarkdownPath);
        }
        fs::write(path, text)?;
        Ok(())
    }

    fn save_untitled_recovery(&self, content: &str) -> AppResult<()> {
        let path = self.paths.untitled_recovery_file();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(())
    }

    fn load_untitled_recovery(&self) -> AppResult<Option<String>> {
        let path = self.paths.untitled_recovery_file();
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&path)?;
        if content.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(content))
    }

    fn clear_untitled_recovery(&self) -> AppResult<()> {
        let path = self.paths.untitled_recovery_file();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}
