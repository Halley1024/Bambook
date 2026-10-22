use super::*;

impl LocalLibraryRepository {
    pub(super) fn load_library(&self) -> AppResult<Vec<StoredDocumentEntry>> {
        let _guard = self
            .library_lock
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        self.read_document_library_unlocked()
    }
}

impl LocalLibraryRepository {
    pub(super) fn relink(
        &self,
        document_id: &str,
        new_path: &str,
    ) -> AppResult<StoredDocumentEntry> {
        validate_document_id(document_id)?;
        let canonical = fs::canonicalize(new_path)?;
        if !canonical.is_file() {
            return Err(AppError::UnsupportedDocumentType);
        }
        let _guard = self
            .library_lock
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        let mut documents = self.read_document_library_unlocked()?;
        let entry = documents
            .iter_mut()
            .find(|entry| entry.id == document_id)
            .ok_or(AppError::UnsupportedDocumentType)?;
        let extension = canonical
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let kind_matches = match entry.kind {
            DocumentKind::Pdf => extension.eq_ignore_ascii_case("pdf"),
            DocumentKind::Markdown => matches!(
                extension.to_ascii_lowercase().as_str(),
                "md" | "markdown" | "mdown" | "mkd"
            ),
        };
        if !kind_matches {
            return Err(AppError::UnsupportedDocumentType);
        }
        let title = canonical
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(AppError::MissingFileName)?
            .to_owned();
        let canonical_string = path_to_user_string(&canonical);
        let is_linked_file = entry.storage_mode == "linked-file";
        entry.title = title.clone();
        if is_linked_file {
            entry.path = canonical_string.clone();
        }
        entry.source_path = canonical_string.clone();
        entry.updated_at = unix_timestamp_millis();
        entry.last_opened_at = entry.updated_at;
        let updated = entry.clone();
        let data_path = if updated.data_path.is_empty() {
            self.paths.pdf_directory(&updated.id)
        } else {
            std::path::PathBuf::from(&updated.data_path)
        };
        let metadata_path = data_path.join("metadata.json");
        if updated.kind == DocumentKind::Pdf {
            let previous: Option<StoredPdfMetadata> = read_optional_json(&metadata_path)?;
            write_pretty_json(
                &metadata_path,
                &StoredPdfMetadata {
                    schema_version: 2,
                    document_id: updated.id.clone(),
                    document_type: "pdf".into(),
                    display_name: title,
                    source_path: canonical_string.clone(),
                    document_path: updated.path.clone(),
                    storage_mode: updated.storage_mode.clone(),
                    content_hash: sha256_file(&canonical)?,
                    created_at: previous
                        .as_ref()
                        .map(|value| value.created_at.clone())
                        .unwrap_or_else(unix_timestamp),
                    updated_at: unix_timestamp(),
                },
            )?;
        } else {
            let previous: Option<StoredMarkdownMetadata> = read_optional_json(&metadata_path)?;
            write_pretty_json(
                &metadata_path,
                &StoredMarkdownMetadata {
                    schema_version: 1,
                    document_id: updated.id.clone(),
                    document_type: "markdown".into(),
                    display_name: title,
                    source_path: canonical_string.clone(),
                    document_path: updated.path.clone(),
                    storage_mode: updated.storage_mode.clone(),
                    created_at: previous
                        .as_ref()
                        .map(|value| value.created_at.clone())
                        .unwrap_or_else(unix_timestamp),
                    updated_at: unix_timestamp(),
                },
            )?;
        }
        self.write_document_library_unlocked(&documents)?;
        Ok(updated)
    }
}

impl LocalLibraryRepository {
    pub(super) fn upsert_pdf_library_entry(&self, managed: &ManagedPdfDocument) -> AppResult<()> {
        let _guard = self
            .library_lock
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        let mut documents = self.read_document_library_unlocked()?;
        let now = unix_timestamp_millis();
        if let Some(entry) = documents
            .iter_mut()
            .find(|entry| entry.id == managed.document_id)
        {
            entry.title = managed.display_name.clone();
            entry.path = managed.storage_path.clone();
            entry.source_path = managed.source_path.clone();
            entry.data_path = managed.data_path.clone();
            entry.storage_mode = managed.storage_mode.clone();
            entry.updated_at = now;
            entry.last_opened_at = now;
        } else {
            documents.push(StoredDocumentEntry {
                id: managed.document_id.clone(),
                title: managed.display_name.clone(),
                kind: DocumentKind::Pdf,
                path: managed.storage_path.clone(),
                source_path: managed.source_path.clone(),
                data_path: managed.data_path.clone(),
                storage_mode: managed.storage_mode.clone(),
                created_at: now,
                updated_at: now,
                last_opened_at: now,
            });
        }
        documents.sort_by(|left, right| right.last_opened_at.cmp(&left.last_opened_at));
        self.write_document_library_unlocked(&documents)
    }

    pub(super) fn upsert_markdown_library_entry(
        &self,
        managed: &ManagedMarkdownDocument,
    ) -> AppResult<()> {
        let _guard = self
            .library_lock
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        let mut documents = self.read_document_library_unlocked()?;
        let now = unix_timestamp_millis();
        if let Some(entry) = documents
            .iter_mut()
            .find(|entry| entry.id == managed.document_id)
        {
            entry.title = managed.display_name.clone();
            entry.path = managed.storage_path.clone();
            entry.source_path = managed.source_path.clone();
            entry.data_path = managed.data_path.clone();
            entry.storage_mode = managed.storage_mode.clone();
            entry.updated_at = now;
            entry.last_opened_at = now;
        } else {
            documents.push(StoredDocumentEntry {
                id: managed.document_id.clone(),
                title: managed.display_name.clone(),
                kind: DocumentKind::Markdown,
                path: managed.storage_path.clone(),
                source_path: managed.source_path.clone(),
                data_path: managed.data_path.clone(),
                storage_mode: managed.storage_mode.clone(),
                created_at: now,
                updated_at: now,
                last_opened_at: now,
            });
        }
        documents.sort_by(|left, right| right.last_opened_at.cmp(&left.last_opened_at));
        self.write_document_library_unlocked(&documents)
    }

    pub(super) fn read_document_library_unlocked(&self) -> AppResult<Vec<StoredDocumentEntry>> {
        let stored: Option<StoredDocumentLibrary> =
            read_optional_json(&self.paths.storage_layout().document_library_file)?;
        Ok(stored
            .unwrap_or_default()
            .documents
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub(super) fn write_document_library_unlocked(
        &self,
        documents: &[StoredDocumentEntry],
    ) -> AppResult<()> {
        let mut documents = documents.to_vec();
        documents.sort_by(|left, right| right.last_opened_at.cmp(&left.last_opened_at));
        let mut ids = HashSet::new();
        let mut paths = HashSet::new();
        documents.retain(|entry| {
            let id = entry.id.to_ascii_lowercase();
            let path = (entry.kind, stored_path_key(&entry.path));
            if ids.contains(&id) || paths.contains(&path) {
                return false;
            }
            ids.insert(id);
            paths.insert(path);
            true
        });
        write_pretty_json(
            &self.paths.storage_layout().document_library_file,
            &StoredDocumentLibrary {
                schema_version: 2,
                documents: documents.iter().map(Into::into).collect(),
            },
        )
    }
}
