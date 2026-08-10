use std::{fs, io::ErrorKind};

use crate::{
    application::ports::DocumentPathInspector, domain::document_history::DocumentAvailability,
};

pub(crate) struct FileDocumentPathInspector;

impl DocumentPathInspector for FileDocumentPathInspector {
    fn inspect(&self, path: &str) -> DocumentAvailability {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => DocumentAvailability::Available,
            Ok(_) => DocumentAvailability::NotFile,
            Err(error) if error.kind() == ErrorKind::NotFound => DocumentAvailability::Missing,
            Err(_) => DocumentAvailability::Inaccessible,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        application::ports::DocumentPathInspector, domain::document_history::DocumentAvailability,
    };

    use super::FileDocumentPathInspector;

    #[test]
    fn distinguishes_files_missing_paths_and_directories() {
        let root =
            std::env::temp_dir().join(format!("bambook-path-inspector-{}", std::process::id()));
        let file = root.join("notes.md");
        fs::create_dir_all(&root).unwrap();
        fs::write(&file, "# Bambook").unwrap();
        let inspector = FileDocumentPathInspector;

        assert_eq!(
            inspector.inspect(file.to_str().unwrap()),
            DocumentAvailability::Available
        );
        assert_eq!(
            inspector.inspect(root.join("missing.pdf").to_str().unwrap()),
            DocumentAvailability::Missing
        );
        assert_eq!(
            inspector.inspect(root.to_str().unwrap()),
            DocumentAvailability::NotFile
        );
        fs::remove_dir_all(root).unwrap();
    }
}
