use super::*;

impl LocalLibraryRepository {
    pub(super) fn initialize_document_library(&self) -> AppResult<()> {
        let _guard = self
            .library_lock
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        let path = &self.paths.storage_layout().document_library_file;
        if path.exists() {
            let documents = self.read_document_library_unlocked()?;
            self.write_document_library_unlocked(&documents)?;
            self.normalize_metadata_paths(&documents)?;
            return Ok(());
        }
        let mut documents = Vec::new();
        for entry in fs::read_dir(&self.paths.storage_layout().storage)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let document_id = entry.file_name().to_string_lossy().into_owned();
            if !document_id.starts_with("pdf-") || validate_document_id(&document_id).is_err() {
                continue;
            }
            let metadata: Option<StoredPdfMetadata> =
                read_optional_json(&self.paths.pdf_metadata_file(&document_id))?;
            let Some(metadata) = metadata else { continue };
            let storage_path = self.paths.pdf_document_file(&document_id);
            if !storage_path.is_file() {
                continue;
            }
            let created_at = parse_metadata_timestamp(&metadata.created_at);
            let updated_at = parse_metadata_timestamp(&metadata.updated_at).max(created_at);
            documents.push(StoredDocumentEntry {
                id: document_id,
                title: metadata.display_name,
                kind: DocumentKind::Pdf,
                path: path_to_user_string(&storage_path),
                source_path: metadata.source_path,
                data_path: path_to_user_string(&entry.path()),
                storage_mode: metadata.storage_mode,
                created_at,
                updated_at,
                last_opened_at: updated_at,
            });
        }
        self.write_document_library_unlocked(&documents)
    }

    pub(super) fn normalize_metadata_paths(
        &self,
        documents: &[StoredDocumentEntry],
    ) -> AppResult<()> {
        for document in documents {
            if document.data_path.is_empty() {
                continue;
            }
            let metadata_path = Path::new(&document.data_path).join("metadata.json");
            match document.kind {
                DocumentKind::Pdf => {
                    if let Ok(Some(mut metadata)) =
                        read_optional_json::<StoredPdfMetadata>(&metadata_path)
                    {
                        metadata.source_path = normalize_stored_path(metadata.source_path);
                        metadata.document_path = normalize_stored_path(metadata.document_path);
                        write_pretty_json(&metadata_path, &metadata)?;
                    }
                }
                DocumentKind::Markdown => {
                    if let Ok(Some(mut metadata)) =
                        read_optional_json::<StoredMarkdownMetadata>(&metadata_path)
                    {
                        metadata.source_path = normalize_stored_path(metadata.source_path);
                        metadata.document_path = normalize_stored_path(metadata.document_path);
                        write_pretty_json(&metadata_path, &metadata)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn migrate_legacy_annotations(
        &self,
        requested_path: &str,
        canonical_path: &str,
        _document_id: &str,
        data_path: &Path,
    ) -> AppResult<()> {
        let destination = data_path.join("annotations.json");
        if destination.exists() {
            return Ok(());
        }
        let legacy_directory = self.paths.storage_layout().storage.join("annotations-json");
        for legacy_id in [
            sha256_bytes(requested_path.as_bytes()),
            sha256_bytes(canonical_path.as_bytes()),
        ] {
            let source = legacy_directory.join(format!("{legacy_id}.json"));
            if source.exists() {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(source, &destination)?;
                break;
            }
        }
        Ok(())
    }
}
