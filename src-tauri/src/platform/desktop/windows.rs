use crate::shared::error::AppResult;
use std::process::Command;
pub(super) fn reveal(path: &str) -> AppResult<()> {
    Command::new("explorer.exe")
        .arg(format!("/select,{path}"))
        .spawn()?;
    Ok(())
}
pub(super) fn open_directory(path: &str) -> AppResult<()> {
    Command::new("explorer.exe").arg(path).spawn()?;
    Ok(())
}
pub(super) fn open_external(url: &str) -> AppResult<()> {
    Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .arg(url)
        .spawn()?;
    Ok(())
}
