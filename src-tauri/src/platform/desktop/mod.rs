pub(crate) mod commands;
#[cfg(not(target_os = "windows"))]
mod linux;
#[cfg(target_os = "windows")]
mod windows;
use crate::shared::error::{AppError, AppResult};
#[cfg(not(target_os = "windows"))]
use linux as os;
#[cfg(target_os = "windows")]
use windows as os;

pub(crate) fn reveal(path: &str) -> AppResult<()> {
    os::reveal(path)
}
pub(crate) fn open_directory(path: &str) -> AppResult<()> {
    os::open_directory(path)
}
pub(crate) fn open_external(url: &str) -> AppResult<()> {
    let target = url.trim();
    if !is_allowed_external_url(target) {
        return Err(AppError::UnsafeExternalLink);
    }
    os::open_external(target)
}
fn is_allowed_external_url(value: &str) -> bool {
    if value.contains('\0') {
        return false;
    }
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
