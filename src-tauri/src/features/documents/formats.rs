use std::path::Path;
pub(crate) fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("md" | "markdown" | "mdown" | "mkd")
    )
}
#[cfg(test)]
mod tests {
    use super::is_markdown_path;
    use std::path::Path;

    #[test]
    fn recognizes_markdown_extensions() {
        assert!(is_markdown_path(Path::new("notes.md")));
        assert!(is_markdown_path(Path::new("README.MARKDOWN")));
        assert!(!is_markdown_path(Path::new("document.pdf")));
    }
}
