use crate::shared::error::{AppError, AppResult};
use std::{path::Path, process::Command};
pub(super) fn reveal(path: &str) -> AppResult<()> {
    let parent = Path::new(path)
        .parent()
        .ok_or(AppError::MissingParentDirectory)?;
    open_directory(&parent.to_string_lossy())
}
pub(super) fn open_directory(path: &str) -> AppResult<()> {
    Command::new("xdg-open").arg(path).spawn()?;
    Ok(())
}
pub(super) fn open_external(url: &str) -> AppResult<()> {
    Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}
