use std::{fs, path::Path};

use crate::{domain::markdown::MarkdownFileEntry, error::AppResult};

const MAX_DIRECTORY_DEPTH: usize = 12;

pub(super) fn collect_markdown_entries(
    directory: &Path,
    depth: usize,
) -> AppResult<Vec<MarkdownFileEntry>> {
    if depth >= MAX_DIRECTORY_DEPTH {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }

        let path = entry.path();
        if file_type.is_dir() {
            let Ok(children) = collect_markdown_entries(&path, depth + 1) else {
                continue;
            };
            if children.is_empty() {
                continue;
            }
            entries.push(MarkdownFileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                is_directory: true,
                children,
            });
        } else if file_type.is_file() && is_markdown_path(&path) {
            entries.push(MarkdownFileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                is_directory: false,
                children: Vec::new(),
            });
        }
    }

    entries.sort_by(|left, right| {
        right
            .is_directory
            .cmp(&left.is_directory)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(entries)
}

pub(super) fn collect_supported_entries(
    directory: &Path,
    depth: usize,
) -> AppResult<Vec<MarkdownFileEntry>> {
    collect_entries(directory, depth, is_supported_document_path)
}

fn collect_entries(
    directory: &Path,
    depth: usize,
    accepts_file: fn(&Path) -> bool,
) -> AppResult<Vec<MarkdownFileEntry>> {
    if depth >= MAX_DIRECTORY_DEPTH {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() { continue; }
        let path = entry.path();
        if file_type.is_dir() {
            let Ok(children) = collect_entries(&path, depth + 1, accepts_file) else { continue };
            if children.is_empty() { continue; }
            entries.push(MarkdownFileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                is_directory: true,
                children,
            });
        } else if file_type.is_file() && accepts_file(&path) {
            entries.push(MarkdownFileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                is_directory: false,
                children: Vec::new(),
            });
        }
    }
    entries.sort_by(|left, right| right.is_directory.cmp(&left.is_directory)
        .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase())));
    Ok(entries)
}

fn is_supported_document_path(path: &Path) -> bool {
    is_markdown_path(path)
        || path.extension().and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("pdf"))
}

pub(super) fn is_markdown_path(path: &Path) -> bool {
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
