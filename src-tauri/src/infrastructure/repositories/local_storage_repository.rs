use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    application::ports::StorageRepository,
    domain::{
        document::DocumentKind,
        settings::{AppSettings, DocumentImportMode},
        storage::{
            ManagedMarkdownDocument, ManagedPdfDocument, PdfBookmark, PdfReadingState, StorageLayout, StoredDocumentEntry,
        },
    },
    error::{AppError, AppResult},
    infrastructure::{
        filesystem::{read_optional_json, write_file_atomically, write_pretty_json},
        paths::{path_to_user_string, validate_document_id, AppPaths},
        utils::sanitize_filename,
    },
};

pub(crate) struct LocalStorageRepository {
    paths: AppPaths,
    library_lock: Mutex<()>,
}

impl LocalStorageRepository {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            library_lock: Mutex::new(()),
        }
    }
}

impl StorageRepository for LocalStorageRepository {
    fn initialize(&self) -> AppResult<StorageLayout> {
        let layout = self.layout();
        for directory in layout.required_directories() {
            fs::create_dir_all(directory).map_err(|source| AppError::StorageIo {
                operation: "directory creation",
                path: directory.to_path_buf(),
                source,
            })?;
            verify_directory_writable(directory)?;
        }
        self.initialize_document_library()?;
        Ok(layout)
    }

    fn layout(&self) -> StorageLayout {
        self.paths.storage_layout()
    }

    fn prepare_pdf(&self, path: &str, settings: &AppSettings) -> AppResult<ManagedPdfDocument> {
        let source = fs::canonicalize(path)?;
        if let Some(document_id) = self.managed_pdf_id(&source) {
            return self.open_managed_pdf(&document_id, &source);
        }
        let documents = self.read_document_library_unlocked()?;
        if let Some(entry) = documents.iter().find(|entry| {
            entry.kind == DocumentKind::Pdf && [entry.path.as_str(), entry.source_path.as_str()].iter()
                .any(|candidate| Path::new(candidate).canonicalize().ok().as_ref() == Some(&source))
        }).cloned() {
            let open_path = if Path::new(&entry.path).is_file() { entry.path.clone() } else { path_to_user_string(&source) };
            return Ok(ManagedPdfDocument { document_id: entry.id, display_name: entry.title, source_path: entry.source_path,
                storage_path: open_path, data_path: entry.data_path, storage_mode: entry.storage_mode });
        }
        let source_path = path_to_user_string(&source);
        let document_id = format!("pdf-{}", sha256_bytes(normalized_path(&source).as_bytes()));
        let display_name = source.file_name().and_then(|name| name.to_str()).ok_or(AppError::MissingFileName)?.to_owned();
        if settings.pdf_import_mode == DocumentImportMode::Link {
            if let Some(entry) = find_linked_document_by_name(&documents, DocumentKind::Pdf, &display_name)
            {
                return Ok(ManagedPdfDocument { document_id: entry.id.clone(), display_name,
                    source_path: source_path.clone(), storage_path: source_path,
                    data_path: entry.data_path.clone(), storage_mode: entry.storage_mode.clone() });
            }
        }
        let data_path = self.paths.pdf_directory_for(&document_id, settings);
        let storage_mode = if settings.pdf_import_mode == DocumentImportMode::Copy { "managed-copy" } else { "linked-file" };
        Ok(ManagedPdfDocument { document_id, display_name, source_path: source_path.clone(), storage_path: source_path,
            data_path: path_to_user_string(&data_path), storage_mode: storage_mode.into() })
    }

    fn persist_pdf(&self, path: &str, settings: &AppSettings) -> AppResult<ManagedPdfDocument> {
        let source = fs::canonicalize(path)?;
        if let Some(document_id) = self.managed_pdf_id(&source) {
            let managed = self.open_managed_pdf(&document_id, &source)?;
            self.upsert_pdf_library_entry(&managed)?;
            return Ok(managed);
        }

        let documents = self.read_document_library_unlocked()?;
        if let Some(entry) = find_document_by_path(&documents, DocumentKind::Pdf, &source) {
            return self.persist_existing_pdf(entry, &source, settings);
        }

        let source_path = path_to_user_string(&source);
        let document_id = format!("pdf-{}", sha256_bytes(normalized_path(&source).as_bytes()));
        let display_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(AppError::MissingFileName)?
            .to_owned();
        if settings.pdf_import_mode == DocumentImportMode::Link {
            if let Some(existing) = find_linked_document_by_name(&documents, DocumentKind::Pdf, &display_name)
                .filter(|entry| entry.id != document_id)
            {
                let updated = self.relink_document(&existing.id, path)?;
                return Ok(ManagedPdfDocument { document_id: updated.id, display_name: updated.title,
                    source_path: updated.source_path, storage_path: updated.path,
                    data_path: updated.data_path, storage_mode: updated.storage_mode });
            }
        }
        if find_document_by_name(&documents, DocumentKind::Pdf, &display_name)
            .is_some_and(|entry| entry.id != document_id)
        {
            return Err(AppError::DuplicateManagedDocumentName { name: display_name });
        }
        let data_path = self.paths.pdf_directory_for(&document_id, settings);
        let managed_copy_path = self.paths.pdf_document_file_for(&document_id, settings);
        let metadata_path = self.paths.pdf_metadata_file_for(&document_id, settings);
        let storage_mode = match settings.pdf_import_mode { DocumentImportMode::Copy => "managed-copy", DocumentImportMode::Link => "linked-file" };
        let storage_path = if settings.pdf_import_mode == DocumentImportMode::Copy { managed_copy_path.clone() } else { source.clone() };
        let content_hash = sha256_file(&source)?;
        let previous: Option<StoredPdfMetadata> = read_optional_json(&metadata_path)?;
        let should_copy = previous
            .as_ref()
            .map(|metadata| metadata.content_hash != content_hash)
            .unwrap_or(true)
            || !managed_copy_path.exists();
        if settings.pdf_import_mode == DocumentImportMode::Copy && should_copy {
            write_file_atomically(&managed_copy_path, |temporary| {
                fs::copy(&source, temporary)?;
                Ok(())
            })?;
        }
        let now = unix_timestamp();
        write_pretty_json(
            &metadata_path,
            &StoredPdfMetadata {
                schema_version: 2,
                document_id: document_id.clone(),
                document_type: "pdf".into(),
                display_name: display_name.clone(),
                source_path: source_path.clone(),
                document_path: path_to_user_string(&storage_path),
                storage_mode: storage_mode.into(),
                content_hash,
                created_at: previous
                    .map(|metadata| metadata.created_at)
                    .unwrap_or_else(|| now.clone()),
                updated_at: now,
            },
        )?;
        self.migrate_legacy_annotations(path, &source_path, &document_id, &data_path)?;
        ensure_json_array(&data_path.join("annotations.json"))?;
        ensure_json_array(&data_path.join("bookmarks.json"))?;
        let managed = ManagedPdfDocument {
            document_id,
            display_name,
            source_path,
            storage_path: path_to_user_string(&storage_path),
            data_path: path_to_user_string(&data_path),
            storage_mode: storage_mode.into(),
        };
        self.upsert_pdf_library_entry(&managed)?;
        Ok(managed)
    }

    fn prepare_markdown(&self, path: &str, settings: &AppSettings) -> AppResult<ManagedMarkdownDocument> {
        let source = fs::canonicalize(path)?;
        let source_path = path_to_user_string(&source);
        let documents = self.read_document_library_unlocked()?;
        if let Some(entry) = documents.iter().find(|entry| {
            entry.kind == DocumentKind::Markdown && [entry.path.as_str(), entry.source_path.as_str()].iter()
                .any(|candidate| Path::new(candidate).canonicalize().ok().as_ref() == Some(&source))
        }).cloned() {
            let open_path = if Path::new(&entry.path).is_file() { entry.path.clone() } else { source_path.clone() };
            return Ok(ManagedMarkdownDocument { document_id: entry.id, display_name: entry.title, source_path: entry.source_path,
                storage_path: open_path, data_path: entry.data_path, storage_mode: entry.storage_mode });
        }
        let document_id = format!("markdown-{}", sha256_bytes(normalized_path(&source).as_bytes()));
        let display_name = source.file_name().and_then(|value| value.to_str()).ok_or(AppError::MissingFileName)?.to_owned();
        if settings.markdown_import_mode == DocumentImportMode::Link {
            if let Some(entry) = find_linked_document_by_name(&documents, DocumentKind::Markdown, &display_name)
            {
                return Ok(ManagedMarkdownDocument { document_id: entry.id.clone(), display_name,
                    source_path: source_path.clone(), storage_path: source_path,
                    data_path: entry.data_path.clone(), storage_mode: entry.storage_mode.clone() });
            }
        }
        let data_path = self.paths.markdown_directory_for(&document_id, settings);
        let storage_mode = if settings.markdown_import_mode == DocumentImportMode::Copy { "managed-copy" } else { "linked-file" };
        Ok(ManagedMarkdownDocument { document_id, display_name, source_path: source_path.clone(), storage_path: source_path,
            data_path: path_to_user_string(&data_path), storage_mode: storage_mode.into() })
    }

    fn persist_markdown(&self, path: &str, settings: &AppSettings) -> AppResult<ManagedMarkdownDocument> {
        let source = fs::canonicalize(path)?;
        let source_path = path_to_user_string(&source);
        let documents = self.read_document_library_unlocked()?;
        if let Some(entry) = find_document_by_path(&documents, DocumentKind::Markdown, &source) {
            return self.persist_existing_markdown(entry, &source, settings);
        }
        let document_id = format!("markdown-{}", sha256_bytes(normalized_path(&source).as_bytes()));
        let display_name = source.file_name().and_then(|value| value.to_str()).ok_or(AppError::MissingFileName)?.to_owned();
        if settings.markdown_import_mode == DocumentImportMode::Link {
            if let Some(existing) = find_linked_document_by_name(&documents, DocumentKind::Markdown, &display_name)
                .filter(|entry| entry.id != document_id)
            {
                let updated = self.relink_document(&existing.id, path)?;
                return Ok(ManagedMarkdownDocument { document_id: updated.id, display_name: updated.title,
                    source_path: updated.source_path, storage_path: updated.path,
                    data_path: updated.data_path, storage_mode: updated.storage_mode });
            }
        }
        if find_document_by_name(&documents, DocumentKind::Markdown, &display_name)
            .is_some_and(|entry| entry.id != document_id)
        {
            return Err(AppError::DuplicateManagedDocumentName { name: display_name });
        }
        let data_path = self.paths.markdown_directory_for(&document_id, settings);
        let copy_path = data_path.join("markdown.md");
        let (storage_path, storage_mode) = if settings.markdown_import_mode == DocumentImportMode::Copy {
            write_file_atomically(&copy_path, |temporary| { fs::copy(&source, temporary)?; Ok(()) })?;
            (copy_path, "managed-copy")
        } else { (source.clone(), "linked-file") };
        fs::create_dir_all(data_path.join("assets"))?;
        let now = unix_timestamp();
        write_pretty_json(&data_path.join("metadata.json"), &StoredMarkdownMetadata {
            schema_version: 1, document_id: document_id.clone(), document_type: "markdown".into(), display_name: display_name.clone(),
            source_path: source_path.clone(), document_path: path_to_user_string(&storage_path), storage_mode: storage_mode.into(),
            created_at: now.clone(), updated_at: now,
        })?;
        let managed = ManagedMarkdownDocument { document_id, display_name, source_path, storage_path: path_to_user_string(&storage_path), data_path: path_to_user_string(&data_path), storage_mode: storage_mode.into() };
        self.upsert_markdown_library_entry(&managed)?;
        Ok(managed)
    }

    fn reconcile_document_storage(&self, document_id: &str, settings: &AppSettings) -> AppResult<()> {
        validate_document_id(document_id)?;
        let entry = match self
            .read_document_library_unlocked()?
            .into_iter()
            .find(|entry| entry.id == document_id)
        {
            Some(entry) => entry,
            // Untitled documents and documents that have never been saved do not
            // have a library record yet, so closing them requires no migration.
            None => return Ok(()),
        };

        // A valid managed copy is authoritative and its path remains stable even
        // when the global setting is later changed back to link mode.
        if entry.storage_mode == "managed-copy" && Path::new(&entry.path).is_file() {
            return Ok(());
        }

        // If neither source nor copy is available, leave the record intact. The
        // save/relink transaction owns the user-facing relocation workflow.
        let source = match fs::canonicalize(&entry.source_path) {
            Ok(path) if path.is_file() => path,
            _ => return Ok(()),
        };
        match entry.kind {
            DocumentKind::Pdf => {
                self.persist_existing_pdf(&entry, &source, settings)?;
            }
            DocumentKind::Markdown => {
                self.persist_existing_markdown(&entry, &source, settings)?;
            }
        }
        Ok(())
    }

    fn load_document_library(&self) -> AppResult<Vec<StoredDocumentEntry>> {
        let _guard = self
            .library_lock
            .lock()
            .map_err(|_| AppError::StateUnavailable)?;
        self.read_document_library_unlocked()
    }


    fn relink_document(&self, document_id: &str, new_path: &str) -> AppResult<StoredDocumentEntry> {
        validate_document_id(document_id)?;
        let canonical = fs::canonicalize(new_path)?;
        if !canonical.is_file() { return Err(AppError::UnsupportedDocumentType); }
        let _guard = self.library_lock.lock().map_err(|_| AppError::StateUnavailable)?;
        let mut documents = self.read_document_library_unlocked()?;
        let entry = documents.iter_mut().find(|entry| entry.id == document_id)
            .ok_or(AppError::UnsupportedDocumentType)?;
        let extension = canonical.extension().and_then(|value| value.to_str()).unwrap_or_default();
        let kind_matches = match entry.kind { DocumentKind::Pdf => extension.eq_ignore_ascii_case("pdf"), DocumentKind::Markdown => matches!(extension.to_ascii_lowercase().as_str(), "md" | "markdown" | "mdown" | "mkd") };
        if !kind_matches { return Err(AppError::UnsupportedDocumentType); }
        let title = canonical.file_name().and_then(|value| value.to_str()).ok_or(AppError::MissingFileName)?.to_owned();
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
        let data_path = if updated.data_path.is_empty() { self.paths.pdf_directory(&updated.id) } else { std::path::PathBuf::from(&updated.data_path) };
        let metadata_path = data_path.join("metadata.json");
        if updated.kind == DocumentKind::Pdf {
            let previous: Option<StoredPdfMetadata> = read_optional_json(&metadata_path)?;
            write_pretty_json(&metadata_path, &StoredPdfMetadata {
                schema_version: 2, document_id: updated.id.clone(), document_type: "pdf".into(),
                display_name: title, source_path: canonical_string.clone(), document_path: updated.path.clone(),
                storage_mode: updated.storage_mode.clone(), content_hash: sha256_file(&canonical)?,
                created_at: previous.as_ref().map(|value| value.created_at.clone()).unwrap_or_else(unix_timestamp),
                updated_at: unix_timestamp(),
            })?;
        } else {
            let previous: Option<StoredMarkdownMetadata> = read_optional_json(&metadata_path)?;
            write_pretty_json(&metadata_path, &StoredMarkdownMetadata {
                schema_version: 1, document_id: updated.id.clone(), document_type: "markdown".into(),
                display_name: title, source_path: canonical_string.clone(), document_path: updated.path.clone(),
                storage_mode: updated.storage_mode.clone(),
                created_at: previous.as_ref().map(|value| value.created_at.clone()).unwrap_or_else(unix_timestamp),
                updated_at: unix_timestamp(),
            })?;
        }
        self.write_document_library_unlocked(&documents)?;
        Ok(updated)
    }

    fn export_document_package(&self, document_id: &str, destination: &str) -> AppResult<String> {
        validate_document_id(document_id)?;
        let entry = self.read_document_library_unlocked()?.into_iter().find(|entry| entry.id == document_id)
            .ok_or(AppError::UnsupportedDocumentType)?;
        let destination = Path::new(destination);
        if !destination.is_dir() { return Err(AppError::MissingParentDirectory); }
        let stem = Path::new(&entry.title).file_stem().and_then(|value| value.to_str()).unwrap_or(&entry.title);
        let base_name = sanitize_filename(stem);
        let mut package = destination.join(&base_name);
        let mut suffix = 2_u32;
        while package.exists() {
            package = destination.join(format!("{base_name} ({suffix})"));
            suffix = suffix.saturating_add(1);
        }
        let source = Path::new(&entry.path);
        if !source.is_file() {
            return Err(AppError::StorageIo {
                operation: "document package export",
                path: source.to_path_buf(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "document source is unavailable"),
            });
        }
        let temporary = destination.join(format!(
            ".{base_name}.bambook-export-{}-{}",
            std::process::id(),
            unix_timestamp_millis(),
        ));
        let package_document_id = if document_id.ends_with("-package") {
            document_id.to_owned()
        } else {
            format!("{document_id}-package")
        };
        let package_file_name = sanitize_filename(&entry.title);
        let package_document_path = package.join(&package_file_name);
        let package_document_path_string = path_to_user_string(&package_document_path);
        let package_path_string = path_to_user_string(&package);
        fs::create_dir_all(&temporary)?;
        let export_result = (|| -> AppResult<()> {
            let data = Path::new(&entry.data_path);
            if data.is_dir() { copy_companion_directory(data, &temporary)?; }
            fs::copy(source, temporary.join(&package_file_name))?;
            rewrite_package_document_ids(&temporary.join("annotations.json"), document_id, &package_document_id)?;
            rewrite_package_document_ids(&temporary.join("bookmarks.json"), document_id, &package_document_id)?;
            let previous: Option<StoredPdfMetadata> = read_optional_json(&temporary.join("metadata.json"))?;
            let now = unix_timestamp();
            write_pretty_json(&temporary.join("metadata.json"), &StoredPdfMetadata {
                schema_version: 2,
                document_id: package_document_id.clone(),
                document_type: "pdf".into(),
                display_name: entry.title.clone(),
                source_path: package_document_path_string.clone(),
                document_path: package_document_path_string.clone(),
                storage_mode: "linked-file".into(),
                content_hash: sha256_file(source)?,
                created_at: previous.map(|metadata| metadata.created_at).unwrap_or_else(|| now.clone()),
                updated_at: now,
            })?;
            fs::rename(&temporary, &package)?;
            Ok(())
        })();
        if export_result.is_err() && temporary.exists() {
            let _ = fs::remove_dir_all(&temporary);
        }
        export_result?;
        let register_result = (|| -> AppResult<()> {
            let _guard = self.library_lock.lock().map_err(|_| AppError::StateUnavailable)?;
            let mut documents = self.read_document_library_unlocked()?;
            let now = unix_timestamp_millis();
            let created_at = documents.iter().find(|item| item.id == package_document_id)
                .map(|item| item.created_at).unwrap_or(now);
            documents.retain(|item| item.id != package_document_id);
            documents.push(StoredDocumentEntry {
                id: package_document_id,
                title: entry.title,
                kind: DocumentKind::Pdf,
                path: package_document_path_string.clone(),
                source_path: package_document_path_string,
                data_path: package_path_string,
                storage_mode: "linked-file".into(),
                created_at,
                updated_at: now,
                last_opened_at: now,
            });
            self.write_document_library_unlocked(&documents)
        })();
        if let Err(error) = register_result {
            let _ = fs::remove_dir_all(&package);
            return Err(error);
        }
        Ok(path_to_user_string(&package))
    }

    fn load_pdf_bookmarks(&self, document_id: &str, source_path: Option<&str>) -> AppResult<Vec<PdfBookmark>> {
        validate_document_id(document_id)?;
        let path = self.companion_read_path(document_id, source_path, "bookmarks.json")?;
        let stored: Option<Vec<StoredPdfBookmark>> = match path { Some(path) => read_optional_json(&path)?, None => None };
        Ok(stored
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn save_pdf_bookmarks(&self, document_id: &str, source_path: Option<&str>, bookmarks: &[PdfBookmark]) -> AppResult<()> {
        validate_document_id(document_id)?;
        let stored: Vec<StoredPdfBookmark> = bookmarks.iter().map(Into::into).collect();
        if let Some(directory) = self.persisted_document_data_directory(document_id)? { write_pretty_json(&directory.join("bookmarks.json"), &stored)?; }
        if let Some(sidecar) = existing_sidecar_path(source_path, "bookmarks.json") { write_pretty_json(&sidecar, &stored)?; }
        Ok(())
    }

    fn load_pdf_reading_state(&self, document_id: &str, source_path: Option<&str>) -> AppResult<Option<PdfReadingState>> {
        validate_document_id(document_id)?;
        let path = self.companion_read_path(document_id, source_path, "reading-state.json")?;
        let stored: Option<StoredPdfReadingState> = match path { Some(path) => read_optional_json(&path)?, None => None };
        Ok(stored.map(Into::into))
    }

    fn save_pdf_reading_state(&self, document_id: &str, source_path: Option<&str>, state: &PdfReadingState) -> AppResult<()> {
        validate_document_id(document_id)?;
        let stored = StoredPdfReadingState::from(state);
        if let Some(directory) = self.persisted_document_data_directory(document_id)? { write_pretty_json(&directory.join("reading-state.json"), &stored)?; }
        if let Some(sidecar) = existing_sidecar_path(source_path, "reading-state.json") { write_pretty_json(&sidecar, &stored)?; }
        Ok(())
    }
}

impl LocalStorageRepository {
    fn initialize_document_library(&self) -> AppResult<()> {
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

    fn normalize_metadata_paths(&self, documents: &[StoredDocumentEntry]) -> AppResult<()> {
        for document in documents {
            if document.data_path.is_empty() { continue; }
            let metadata_path = Path::new(&document.data_path).join("metadata.json");
            match document.kind {
                DocumentKind::Pdf => {
                    if let Ok(Some(mut metadata)) = read_optional_json::<StoredPdfMetadata>(&metadata_path) {
                        metadata.source_path = normalize_stored_path(metadata.source_path);
                        metadata.document_path = normalize_stored_path(metadata.document_path);
                        write_pretty_json(&metadata_path, &metadata)?;
                    }
                }
                DocumentKind::Markdown => {
                    if let Ok(Some(mut metadata)) = read_optional_json::<StoredMarkdownMetadata>(&metadata_path) {
                        metadata.source_path = normalize_stored_path(metadata.source_path);
                        metadata.document_path = normalize_stored_path(metadata.document_path);
                        write_pretty_json(&metadata_path, &metadata)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn persist_existing_pdf(&self, entry: &StoredDocumentEntry, source: &Path, settings: &AppSettings) -> AppResult<ManagedPdfDocument> {
        let data_path = existing_data_path(entry, || self.paths.pdf_directory_for(&entry.id, settings));
        let stable_copy_path = if entry.storage_mode == "managed-copy" && !entry.path.is_empty() {
            PathBuf::from(&entry.path)
        } else {
            data_path.join("document.pdf")
        };
        let metadata_path = data_path.join("metadata.json");
        let previous: Option<StoredPdfMetadata> = read_optional_json(&metadata_path)?;
        let content_hash = sha256_file(source)?;
        let is_document_package = entry.id.ends_with("-package");
        let copy_available = entry.storage_mode == "managed-copy" && stable_copy_path.is_file();
        let use_managed_copy = !is_document_package
            && (copy_available || settings.pdf_import_mode == DocumentImportMode::Copy);
        let copy_needs_refresh = !copy_available || (settings.pdf_import_mode == DocumentImportMode::Copy
            && previous.as_ref().is_none_or(|metadata| metadata.content_hash != content_hash));
        if use_managed_copy && copy_needs_refresh {
            write_file_atomically(&stable_copy_path, |temporary| { fs::copy(source, temporary)?; Ok(()) })?;
        }
        let source_path = path_to_user_string(source);
        let storage_path = if use_managed_copy { stable_copy_path } else { source.to_path_buf() };
        let storage_mode = if use_managed_copy { "managed-copy" } else { "linked-file" };
        let now = unix_timestamp();
        write_pretty_json(&metadata_path, &StoredPdfMetadata {
            schema_version: 2, document_id: entry.id.clone(), document_type: "pdf".into(),
            display_name: entry.title.clone(), source_path: source_path.clone(),
            document_path: path_to_user_string(&storage_path), storage_mode: storage_mode.into(),
            content_hash,
            created_at: previous.as_ref().map(|value| value.created_at.clone()).unwrap_or_else(|| now.clone()),
            updated_at: now,
        })?;
        ensure_json_array(&data_path.join("annotations.json"))?;
        ensure_json_array(&data_path.join("bookmarks.json"))?;
        let managed = ManagedPdfDocument { document_id: entry.id.clone(), display_name: entry.title.clone(), source_path,
            storage_path: path_to_user_string(&storage_path), data_path: path_to_user_string(&data_path), storage_mode: storage_mode.into() };
        self.upsert_pdf_library_entry(&managed)?;
        Ok(managed)
    }

    fn persist_existing_markdown(&self, entry: &StoredDocumentEntry, source: &Path, settings: &AppSettings) -> AppResult<ManagedMarkdownDocument> {
        let data_path = existing_data_path(entry, || self.paths.markdown_directory_for(&entry.id, settings));
        let stable_copy_path = if entry.storage_mode == "managed-copy" && !entry.path.is_empty() {
            PathBuf::from(&entry.path)
        } else {
            data_path.join("markdown.md")
        };
        let source_is_copy = entry.storage_mode == "managed-copy" && paths_resolve_to_same_file(&entry.path, source);
        let copy_available = entry.storage_mode == "managed-copy" && stable_copy_path.is_file();
        let use_managed_copy = copy_available || settings.markdown_import_mode == DocumentImportMode::Copy;
        if use_managed_copy && !source_is_copy {
            write_file_atomically(&stable_copy_path, |temporary| { fs::copy(source, temporary)?; Ok(()) })?;
        }
        fs::create_dir_all(data_path.join("assets"))?;
        let source_path = if source_is_copy { entry.source_path.clone() } else { path_to_user_string(source) };
        let storage_path = if use_managed_copy { stable_copy_path } else { source.to_path_buf() };
        let storage_mode = if use_managed_copy { "managed-copy" } else { "linked-file" };
        let metadata_path = data_path.join("metadata.json");
        let previous: Option<StoredMarkdownMetadata> = read_optional_json(&metadata_path)?;
        let now = unix_timestamp();
        write_pretty_json(&metadata_path, &StoredMarkdownMetadata {
            schema_version: 1, document_id: entry.id.clone(), document_type: "markdown".into(),
            display_name: entry.title.clone(), source_path: source_path.clone(),
            document_path: path_to_user_string(&storage_path), storage_mode: storage_mode.into(),
            created_at: previous.as_ref().map(|value| value.created_at.clone()).unwrap_or_else(|| now.clone()),
            updated_at: now,
        })?;
        let managed = ManagedMarkdownDocument { document_id: entry.id.clone(), display_name: entry.title.clone(), source_path,
            storage_path: path_to_user_string(&storage_path), data_path: path_to_user_string(&data_path), storage_mode: storage_mode.into() };
        self.upsert_markdown_library_entry(&managed)?;
        Ok(managed)
    }

    fn upsert_pdf_library_entry(&self, managed: &ManagedPdfDocument) -> AppResult<()> {
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

    fn upsert_markdown_library_entry(&self, managed: &ManagedMarkdownDocument) -> AppResult<()> {
        let _guard = self.library_lock.lock().map_err(|_| AppError::StateUnavailable)?;
        let mut documents = self.read_document_library_unlocked()?;
        let now = unix_timestamp_millis();
        if let Some(entry) = documents.iter_mut().find(|entry| entry.id == managed.document_id) {
            entry.title = managed.display_name.clone(); entry.path = managed.storage_path.clone();
            entry.source_path = managed.source_path.clone(); entry.data_path = managed.data_path.clone();
            entry.storage_mode = managed.storage_mode.clone(); entry.updated_at = now; entry.last_opened_at = now;
        } else {
            documents.push(StoredDocumentEntry { id: managed.document_id.clone(), title: managed.display_name.clone(), kind: DocumentKind::Markdown,
                path: managed.storage_path.clone(), source_path: managed.source_path.clone(), data_path: managed.data_path.clone(), storage_mode: managed.storage_mode.clone(),
                created_at: now, updated_at: now, last_opened_at: now });
        }
        documents.sort_by(|left, right| right.last_opened_at.cmp(&left.last_opened_at));
        self.write_document_library_unlocked(&documents)
    }

    fn read_document_library_unlocked(&self) -> AppResult<Vec<StoredDocumentEntry>> {
        let stored: Option<StoredDocumentLibrary> =
            read_optional_json(&self.paths.storage_layout().document_library_file)?;
        Ok(stored
            .unwrap_or_default()
            .documents
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn write_document_library_unlocked(&self, documents: &[StoredDocumentEntry]) -> AppResult<()> {
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

    fn migrate_legacy_annotations(
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

    fn managed_pdf_id(&self, path: &Path) -> Option<String> {
        if path.file_name()?.to_string_lossy().to_ascii_lowercase() != "document.pdf" {
            return None;
        }
        let document_id = path.parent()?.file_name()?.to_string_lossy().into_owned();
        validate_document_id(&document_id).ok()?;
        path.parent()?.join("metadata.json").is_file().then_some(document_id)
    }

    fn open_managed_pdf(
        &self,
        document_id: &str,
        storage_path: &Path,
    ) -> AppResult<ManagedPdfDocument> {
        validate_document_id(document_id)?;
        let data_path = storage_path.parent().ok_or(AppError::MissingFileName)?;
        let metadata: Option<StoredPdfMetadata> = read_optional_json(&data_path.join("metadata.json"))?;
        let display_name = metadata
            .as_ref()
            .map(|value| value.display_name.clone())
            .unwrap_or_else(|| "document.pdf".into());
        let source_path = metadata
            .as_ref()
            .map(|value| normalize_stored_path(value.source_path.clone()))
            .unwrap_or_else(|| path_to_user_string(storage_path));
        Ok(ManagedPdfDocument {
            document_id: document_id.to_owned(),
            display_name,
            source_path,
            storage_path: path_to_user_string(storage_path),
            data_path: path_to_user_string(data_path),
            storage_mode: metadata.as_ref().map(|value| value.storage_mode.clone()).unwrap_or_else(|| "managed-copy".into()),
        })
    }

    fn document_data_directory(&self, document_id: &str) -> AppResult<std::path::PathBuf> {
        validate_document_id(document_id)?;
        Ok(self.read_document_library_unlocked()?.into_iter()
            .find(|entry| entry.id == document_id)
            .filter(|entry| !entry.data_path.is_empty())
            .map(|entry| std::path::PathBuf::from(entry.data_path))
            .unwrap_or_else(|| self.paths.pdf_directory(document_id)))
    }

    fn persisted_document_data_directory(&self, document_id: &str) -> AppResult<Option<std::path::PathBuf>> {
        validate_document_id(document_id)?;
        let indexed = self.read_document_library_unlocked()?.into_iter().find(|entry| entry.id == document_id);
        if let Some(entry) = indexed {
            let directory = if entry.data_path.is_empty() { self.paths.pdf_directory(document_id) } else { entry.data_path.into() };
            return Ok(Some(directory));
        }
        let legacy = self.paths.pdf_directory(document_id);
        Ok(legacy.is_dir().then_some(legacy))
    }

    fn companion_read_path(&self, document_id: &str, source_path: Option<&str>, file_name: &str) -> AppResult<Option<std::path::PathBuf>> {
        if let Some(sidecar) = existing_sidecar_path(source_path, file_name) { return Ok(Some(sidecar)); }
        let managed = self.document_data_directory(document_id)?.join(file_name);
        Ok(managed.is_file().then_some(managed))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredPdfMetadata {
    schema_version: u32,
    document_id: String,
    #[serde(rename = "type")]
    document_type: String,
    display_name: String,
    source_path: String,
    #[serde(default)]
    document_path: String,
    storage_mode: String,
    content_hash: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredMarkdownMetadata {
    schema_version: u32,
    document_id: String,
    #[serde(rename = "type")]
    document_type: String,
    display_name: String,
    source_path: String,
    document_path: String,
    storage_mode: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredDocumentLibrary {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    documents: Vec<StoredDocumentLibraryEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredDocumentLibraryEntry {
    id: String,
    title: String,
    kind: StoredDocumentKind,
    path: String,
    source_path: String,
    #[serde(default)]
    data_path: String,
    #[serde(default = "default_managed_copy")]
    storage_mode: String,
    created_at: u64,
    updated_at: u64,
    last_opened_at: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum StoredDocumentKind {
    Pdf,
    Markdown,
}

impl From<StoredDocumentLibraryEntry> for StoredDocumentEntry {
    fn from(value: StoredDocumentLibraryEntry) -> Self {
        Self {
            id: value.id,
            title: value.title,
            kind: match value.kind {
                StoredDocumentKind::Pdf => DocumentKind::Pdf,
                StoredDocumentKind::Markdown => DocumentKind::Markdown,
            },
            path: normalize_stored_path(value.path),
            source_path: normalize_stored_path(value.source_path),
            data_path: if value.data_path.is_empty() { String::new() } else { normalize_stored_path(value.data_path) },
            storage_mode: value.storage_mode,
            created_at: value.created_at,
            updated_at: value.updated_at,
            last_opened_at: value.last_opened_at,
        }
    }
}

impl From<&StoredDocumentEntry> for StoredDocumentLibraryEntry {
    fn from(value: &StoredDocumentEntry) -> Self {
        Self {
            id: value.id.clone(),
            title: value.title.clone(),
            kind: match value.kind {
                DocumentKind::Pdf => StoredDocumentKind::Pdf,
                DocumentKind::Markdown => StoredDocumentKind::Markdown,
            },
            path: value.path.clone(),
            source_path: value.source_path.clone(),
            data_path: value.data_path.clone(),
            storage_mode: value.storage_mode.clone(),
            created_at: value.created_at,
            updated_at: value.updated_at,
            last_opened_at: value.last_opened_at,
        }
    }
}

fn default_managed_copy() -> String { "managed-copy".into() }
fn default_bookmark_color() -> String { "#26765A".into() }

fn find_document_by_name<'a>(documents: &'a [StoredDocumentEntry], kind: DocumentKind, display_name: &str) -> Option<&'a StoredDocumentEntry> {
    documents.iter().find(|entry| entry.kind == kind && entry.title.eq_ignore_ascii_case(display_name))
}

fn find_document_by_path<'a>(documents: &'a [StoredDocumentEntry], kind: DocumentKind, path: &Path) -> Option<&'a StoredDocumentEntry> {
    documents.iter().find(|entry| entry.kind == kind && [entry.path.as_str(), entry.source_path.as_str()]
        .iter().any(|candidate| paths_resolve_to_same_file(candidate, path)))
}

fn find_linked_document_by_name<'a>(documents: &'a [StoredDocumentEntry], kind: DocumentKind, display_name: &str) -> Option<&'a StoredDocumentEntry> {
    documents.iter().find(|entry| entry.kind == kind && entry.storage_mode == "linked-file"
        && entry.title.eq_ignore_ascii_case(display_name))
}

fn paths_resolve_to_same_file(candidate: &str, path: &Path) -> bool {
    Path::new(candidate).canonicalize().ok().as_ref() == Some(&path.to_path_buf())
}

fn existing_data_path(entry: &StoredDocumentEntry, fallback: impl FnOnce() -> PathBuf) -> PathBuf {
    if entry.data_path.is_empty() { fallback() } else { PathBuf::from(&entry.data_path) }
}

fn normalize_stored_path(value: String) -> String {
    path_to_user_string(Path::new(&value))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredPdfBookmark {
    id: String,
    document_id: String,
    page: u32,
    x: f64,
    y: f64,
    title: String,
    #[serde(default = "default_bookmark_color")]
    color: String,
    created_at: String,
}

impl From<StoredPdfBookmark> for PdfBookmark {
    fn from(value: StoredPdfBookmark) -> Self {
        Self {
            id: value.id,
            document_id: value.document_id,
            page: value.page,
            x: value.x,
            y: value.y,
            title: value.title,
            color: value.color,
            created_at: value.created_at,
        }
    }
}

impl From<&PdfBookmark> for StoredPdfBookmark {
    fn from(value: &PdfBookmark) -> Self {
        Self {
            id: value.id.clone(),
            document_id: value.document_id.clone(),
            page: value.page,
            x: value.x,
            y: value.y,
            title: value.title.clone(),
            color: value.color.clone(),
            created_at: value.created_at.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredPdfReadingState {
    page: u32,
    page_y: Option<f64>,
    zoom_mode: String,
    scale: f64,
    updated_at: String,
}

impl From<StoredPdfReadingState> for PdfReadingState {
    fn from(value: StoredPdfReadingState) -> Self {
        Self {
            page: value.page,
            page_y: value.page_y,
            zoom_mode: value.zoom_mode,
            scale: value.scale,
            updated_at: value.updated_at,
        }
    }
}

impl From<&PdfReadingState> for StoredPdfReadingState {
    fn from(value: &PdfReadingState) -> Self {
        Self {
            page: value.page,
            page_y: value.page_y,
            zoom_mode: value.zoom_mode.clone(),
            scale: value.scale,
            updated_at: value.updated_at.clone(),
        }
    }
}

fn ensure_json_array(path: &Path) -> AppResult<()> {
    if !path.exists() {
        write_pretty_json(path, &Vec::<serde_json::Value>::new())?;
    }
    Ok(())
}

fn existing_sidecar_path(source_path: Option<&str>, file_name: &str) -> Option<std::path::PathBuf> {
    source_path.and_then(|path| Path::new(path).parent()).map(|parent| parent.join(file_name)).filter(|path| path.is_file())
}

fn copy_companion_directory(source: &Path, destination: &Path) -> AppResult<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        let lower_name = name.to_string_lossy().to_ascii_lowercase();
        if matches!(lower_name.as_str(), "document.pdf" | "markdown.md") { continue; }
        let target = destination.join(&name);
        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&target)?;
            copy_companion_directory(&entry.path(), &target)?;
        } else if entry.file_type()?.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn rewrite_package_document_ids(path: &Path, previous_id: &str, package_id: &str) -> AppResult<()> {
    if !path.is_file() { return Ok(()); }
    let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    rewrite_json_document_ids(&mut value, previous_id, package_id);
    write_pretty_json(path, &value)
}

fn rewrite_json_document_ids(value: &mut serde_json::Value, previous_id: &str, package_id: &str) {
    match value {
        serde_json::Value::Object(object) => {
            if object.get("documentId").and_then(serde_json::Value::as_str) == Some(previous_id) {
                object.insert("documentId".into(), serde_json::Value::String(package_id.into()));
            }
            object.values_mut().for_each(|value| rewrite_json_document_ids(value, previous_id, package_id));
        }
        serde_json::Value::Array(items) => items.iter_mut()
            .for_each(|value| rewrite_json_document_ids(value, previous_id, package_id)),
        _ => {}
    }
}

fn stored_path_key(path: &str) -> String {
    path.trim().trim_start_matches(r"\\?\").replace('/', "\\").trim_end_matches('\\').to_ascii_lowercase()
}

fn normalized_path(path: &Path) -> String {
    let value = path_to_user_string(path).replace('\\', "/");
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value
    }
}

fn sha256_file(path: &Path) -> AppResult<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn unix_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn parse_metadata_timestamp(value: &str) -> u64 {
    value
        .parse::<u64>()
        .unwrap_or_default()
        .saturating_mul(1000)
}

fn verify_directory_writable(directory: &Path) -> AppResult<()> {
    let probe = directory.join(format!(
        ".bambook-write-probe-{}-{}",
        std::process::id(),
        next_probe_id()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe)?;
        file.write_all(b"Bambook storage probe")?;
        file.sync_all()?;
        drop(file);
        fs::remove_file(&probe)
    })();

    if let Err(source) = result {
        let _ = fs::remove_file(&probe);
        return Err(AppError::StorageIo {
            operation: "writability check",
            path: directory.to_path_buf(),
            source,
        });
    }
    Ok(())
}

fn next_probe_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        application::ports::StorageRepository,
        domain::{settings::{AppSettings, DocumentImportMode}, storage::{PdfBookmark, PdfReadingState}},
        error::AppError,
    };

    use super::{path_to_user_string, AppPaths, LocalStorageRepository};

    #[test]
    fn initializes_every_required_directory() {
        let root = test_root("initialize");
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        let layout = repository.initialize().unwrap();

        for directory in layout.required_directories() {
            assert!(directory.is_dir(), "missing {}", directory.display());
        }
        assert!(layout.document_library_file.is_file());
        assert!(!root.join(".bambook-write-probe").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imports_pdf_and_persists_its_companion_data_in_one_directory() {
        let root = test_root("managed-pdf");
        let source_directory = root.join("incoming");
        fs::create_dir_all(&source_directory).unwrap();
        let source = source_directory.join("guide.pdf");
        fs::write(&source, b"pdf fixture").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();

        let staged = repository.prepare_pdf(source.to_str().unwrap(), &AppSettings::default()).unwrap();
        let document_directory = root.join("storage").join(&staged.document_id);
        assert_eq!(staged.display_name, "guide.pdf");
        assert!(!document_directory.exists());
        assert!(repository.load_document_library().unwrap().is_empty());
        repository.save_pdf_reading_state(&staged.document_id, None, &PdfReadingState {
            page: 1, page_y: None, zoom_mode: "fit-width".into(), scale: 1.0, updated_at: "now".into(),
        }).unwrap();
        assert!(!document_directory.exists(), "background reading-state save must not create storage");

        let managed = repository.persist_pdf(source.to_str().unwrap(), &AppSettings::default()).unwrap();
        assert!(document_directory.join("document.pdf").is_file());
        assert!(document_directory.join("metadata.json").is_file());
        assert!(document_directory.join("annotations.json").is_file());
        assert!(document_directory.join("bookmarks.json").is_file());
        let library = repository.load_document_library().unwrap();
        assert_eq!(library.len(), 1);
        assert_eq!(library[0].id, managed.document_id);
        assert_eq!(library[0].title, "guide.pdf");

        let bookmark = PdfBookmark {
            id: "bookmark-1".into(),
            document_id: managed.document_id.clone(),
            page: 2,
            x: 0.1,
            y: 0.4,
            title: "Chapter".into(),
            color: "#26765A".into(),
            created_at: "now".into(),
        };
        repository
            .save_pdf_bookmarks(&managed.document_id, None, std::slice::from_ref(&bookmark))
            .unwrap();
        assert_eq!(
            repository.load_pdf_bookmarks(&managed.document_id, None).unwrap(),
            vec![bookmark.clone()]
        );
        fs::write(source_directory.join("bookmarks.json"), format!(
            "[{{\"id\":\"local\",\"documentId\":\"{}\",\"page\":7,\"x\":0.2,\"y\":0.3,\"title\":\"Local sidecar\",\"createdAt\":\"now\"}}]",
            managed.document_id
        )).unwrap();
        let local_bookmarks = repository.load_pdf_bookmarks(&managed.document_id, source.to_str()).unwrap();
        assert_eq!(local_bookmarks[0].title, "Local sidecar");
        fs::remove_file(source_directory.join("bookmarks.json")).unwrap();
        assert_eq!(repository.load_pdf_bookmarks(&managed.document_id, source.to_str()).unwrap(), vec![bookmark]);

        let reading_state = PdfReadingState {
            page: 2,
            page_y: Some(0.4),
            zoom_mode: "fit-width".into(),
            scale: 1.25,
            updated_at: "now".into(),
        };
        repository
            .save_pdf_reading_state(&managed.document_id, None, &reading_state)
            .unwrap();
        assert_eq!(
            repository
                .load_pdf_reading_state(&managed.document_id, None)
                .unwrap(),
            Some(reading_state)
        );
        let export_root = root.join("exports");
        fs::create_dir_all(&export_root).unwrap();
        let package = repository.export_document_package(&managed.document_id, export_root.to_str().unwrap()).unwrap();
        let package = PathBuf::from(package);
        assert_eq!(package.file_name().unwrap(), "guide");
        assert!(package.join("guide.pdf").is_file());
        assert!(package.join("annotations.json").is_file());
        assert!(package.join("bookmarks.json").is_file());
        assert!(package.join("reading-state.json").is_file());
        assert!(package.join("metadata.json").is_file());
        assert!(!package.join("document.pdf").exists());
        let library = repository.load_document_library().unwrap();
        assert_eq!(library.len(), 2);
        let packaged = library.iter().find(|entry| entry.id == format!("{}-package", managed.document_id)).unwrap();
        assert_eq!(packaged.storage_mode, "linked-file");
        assert_eq!(packaged.data_path, path_to_user_string(&package));
        assert_eq!(packaged.path, path_to_user_string(&package.join("guide.pdf")));
        assert!(!root.join("storage").join(&packaged.id).exists());
        let package_metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(package.join("metadata.json")).unwrap(),
        ).unwrap();
        assert_eq!(package_metadata["documentId"], packaged.id);
        assert_eq!(package_metadata["storageMode"], "linked-file");
        let second_package = PathBuf::from(repository.export_document_package(
            &managed.document_id,
            export_root.to_str().unwrap(),
        ).unwrap());
        assert_ne!(second_package, package);
        let library = repository.load_document_library().unwrap();
        assert_eq!(library.len(), 2);
        let packaged = library.iter().find(|entry| entry.id == format!("{}-package", managed.document_id)).unwrap();
        assert_eq!(packaged.data_path, path_to_user_string(&second_package));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn link_mode_keeps_the_source_in_place_and_creates_companion_metadata() {
        let root = test_root("linked-pdf");
        let source = root.join("incoming").join("linked.pdf");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, b"pdf fixture").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        let settings = AppSettings { pdf_import_mode: DocumentImportMode::Link, ..AppSettings::default() };

        let staged = repository.prepare_pdf(source.to_str().unwrap(), &settings).unwrap();
        let data_directory = root.join("storage").join(&staged.document_id);
        assert!(!data_directory.exists());
        assert!(repository.load_document_library().unwrap().is_empty());
        let managed = repository.persist_pdf(source.to_str().unwrap(), &settings).unwrap();
        assert_eq!(managed.storage_path, path_to_user_string(&fs::canonicalize(&source).unwrap()));
        assert!(!data_directory.join("document.pdf").exists());
        assert!(data_directory.join("metadata.json").is_file());
        assert_eq!(repository.load_document_library().unwrap()[0].storage_mode, "linked-file");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relinking_a_managed_copy_preserves_its_managed_document_path() {
        let root = test_root("relink-managed-copy");
        let source = root.join("incoming").join("guide.pdf");
        let relocated = root.join("relocated").join("guide.pdf");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::create_dir_all(relocated.parent().unwrap()).unwrap();
        fs::write(&source, b"pdf fixture").unwrap();
        fs::write(&relocated, b"pdf fixture relocated").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();

        let managed = repository.persist_pdf(source.to_str().unwrap(), &AppSettings::default()).unwrap();
        let original_storage_path = managed.storage_path;
        let updated = repository.relink_document(&managed.document_id, relocated.to_str().unwrap()).unwrap();

        assert_eq!(updated.storage_mode, "managed-copy");
        assert_eq!(updated.path, original_storage_path);
        assert_eq!(updated.source_path, path_to_user_string(&fs::canonicalize(&relocated).unwrap()));
        assert!(PathBuf::from(&updated.path).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn markdown_storage_is_created_only_when_the_document_is_saved() {
        let root = test_root("deferred-markdown");
        let source = root.join("incoming").join("notes.md");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "# Notes").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        let settings = AppSettings::default();

        let staged = repository.prepare_markdown(source.to_str().unwrap(), &settings).unwrap();
        let data_directory = root.join("storage").join(&staged.document_id);
        assert!(!data_directory.exists());
        assert!(repository.load_document_library().unwrap().is_empty());

        repository.persist_markdown(source.to_str().unwrap(), &settings).unwrap();
        assert!(data_directory.join("markdown.md").is_file());
        assert!(data_directory.join("metadata.json").is_file());
        assert!(data_directory.join("assets").is_dir());
        assert_eq!(repository.load_document_library().unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn linked_pdf_with_the_same_name_reuses_the_record_and_relinks_its_source() {
        let root = test_root("linked-pdf-name-deduplication");
        let first_source = root.join("first").join("guide.pdf");
        let relocated_source = root.join("relocated").join("guide.pdf");
        fs::create_dir_all(first_source.parent().unwrap()).unwrap();
        fs::create_dir_all(relocated_source.parent().unwrap()).unwrap();
        fs::write(&first_source, b"first pdf").unwrap();
        fs::write(&relocated_source, b"relocated pdf").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        let settings = AppSettings { pdf_import_mode: DocumentImportMode::Link, ..AppSettings::default() };

        let first = repository.persist_pdf(first_source.to_str().unwrap(), &settings).unwrap();
        let staged = repository.prepare_pdf(relocated_source.to_str().unwrap(), &settings).unwrap();
        assert_eq!(staged.document_id, first.document_id);
        let updated = repository.persist_pdf(relocated_source.to_str().unwrap(), &settings).unwrap();
        assert_eq!(updated.document_id, first.document_id);

        let library = repository.load_document_library().unwrap();
        assert_eq!(library.len(), 1);
        let relocated = path_to_user_string(&fs::canonicalize(&relocated_source).unwrap());
        assert_eq!(library[0].source_path, relocated);
        assert_eq!(library[0].path, relocated);
        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(PathBuf::from(&first.data_path).join("metadata.json")).unwrap(),
        ).unwrap();
        assert_eq!(metadata["sourcePath"], relocated);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn managed_pdf_rejects_a_different_source_with_the_same_name() {
        let root = test_root("managed-pdf-name-collision");
        let first_source = root.join("first").join("guide.pdf");
        let duplicate_source = root.join("duplicate").join("guide.pdf");
        fs::create_dir_all(first_source.parent().unwrap()).unwrap();
        fs::create_dir_all(duplicate_source.parent().unwrap()).unwrap();
        fs::write(&first_source, b"first pdf").unwrap();
        fs::write(&duplicate_source, b"different pdf").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        repository.persist_pdf(first_source.to_str().unwrap(), &AppSettings::default()).unwrap();

        let error = repository.persist_pdf(duplicate_source.to_str().unwrap(), &AppSettings::default()).unwrap_err();
        assert!(matches!(error, AppError::DuplicateManagedDocumentName { name } if name == "guide.pdf"));
        assert_eq!(repository.load_document_library().unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn linked_markdown_with_the_same_name_reuses_the_record_and_relinks_its_source() {
        let root = test_root("linked-markdown-name-deduplication");
        let first_source = root.join("first").join("notes.md");
        let relocated_source = root.join("relocated").join("notes.md");
        fs::create_dir_all(first_source.parent().unwrap()).unwrap();
        fs::create_dir_all(relocated_source.parent().unwrap()).unwrap();
        fs::write(&first_source, "first").unwrap();
        fs::write(&relocated_source, "relocated").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        let settings = AppSettings { markdown_import_mode: DocumentImportMode::Link, ..AppSettings::default() };

        let first = repository.persist_markdown(first_source.to_str().unwrap(), &settings).unwrap();
        let staged = repository.prepare_markdown(relocated_source.to_str().unwrap(), &settings).unwrap();
        assert_eq!(staged.document_id, first.document_id);
        let updated = repository.persist_markdown(relocated_source.to_str().unwrap(), &settings).unwrap();
        assert_eq!(updated.document_id, first.document_id);
        let library = repository.load_document_library().unwrap();
        assert_eq!(library.len(), 1);
        let relocated = path_to_user_string(&fs::canonicalize(&relocated_source).unwrap());
        assert_eq!(library[0].source_path, relocated);
        assert_eq!(library[0].path, relocated);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn managed_markdown_rejects_a_different_source_with_the_same_name() {
        let root = test_root("managed-markdown-name-collision");
        let first_source = root.join("first").join("notes.md");
        let duplicate_source = root.join("duplicate").join("notes.md");
        fs::create_dir_all(first_source.parent().unwrap()).unwrap();
        fs::create_dir_all(duplicate_source.parent().unwrap()).unwrap();
        fs::write(&first_source, "first").unwrap();
        fs::write(&duplicate_source, "different").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        repository.persist_markdown(first_source.to_str().unwrap(), &AppSettings::default()).unwrap();

        let error = repository.persist_markdown(duplicate_source.to_str().unwrap(), &AppSettings::default()).unwrap_err();
        assert!(matches!(error, AppError::DuplicateManagedDocumentName { name } if name == "notes.md"));
        assert_eq!(repository.load_document_library().unwrap().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pdf_storage_mode_upgrades_to_copy_and_only_falls_back_when_the_copy_is_missing() {
        let root = test_root("pdf-storage-mode-transition");
        let source = root.join("source").join("guide.pdf");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, b"pdf fixture").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        let link_settings = AppSettings { pdf_import_mode: DocumentImportMode::Link, ..AppSettings::default() };
        let copy_settings = AppSettings::default();

        let linked = repository.persist_pdf(source.to_str().unwrap(), &link_settings).unwrap();
        assert_eq!(linked.storage_mode, "linked-file");
        let copied = repository.persist_pdf(source.to_str().unwrap(), &copy_settings).unwrap();
        assert_eq!(copied.document_id, linked.document_id);
        assert_eq!(copied.storage_mode, "managed-copy");
        assert!(PathBuf::from(&copied.storage_path).is_file());

        let stable = repository.persist_pdf(source.to_str().unwrap(), &link_settings).unwrap();
        assert_eq!(stable.storage_mode, "managed-copy");
        assert_eq!(stable.storage_path, copied.storage_path);

        fs::remove_file(&stable.storage_path).unwrap();
        let fallback = repository.persist_pdf(source.to_str().unwrap(), &link_settings).unwrap();
        assert_eq!(fallback.storage_mode, "linked-file");
        assert_eq!(fallback.storage_path, path_to_user_string(&fs::canonicalize(&source).unwrap()));
        let restored = repository.persist_pdf(source.to_str().unwrap(), &copy_settings).unwrap();
        assert_eq!(restored.storage_mode, "managed-copy");
        assert!(PathBuf::from(&restored.storage_path).is_file());

        let library = repository.load_document_library().unwrap();
        assert_eq!(library.len(), 1);
        assert_eq!(library[0].storage_mode, "managed-copy");
        assert_eq!(library[0].path, restored.storage_path);
        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(PathBuf::from(&restored.data_path).join("metadata.json")).unwrap(),
        ).unwrap();
        assert_eq!(metadata["storageMode"], "managed-copy");
        assert_eq!(metadata["documentPath"], restored.storage_path);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn close_reconciliation_applies_the_current_mode_without_replacing_a_valid_copy() {
        let root = test_root("close-storage-reconciliation");
        let source = root.join("source").join("guide.pdf");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, b"pdf fixture").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        let link_settings = AppSettings { pdf_import_mode: DocumentImportMode::Link, ..AppSettings::default() };
        let copy_settings = AppSettings::default();

        let linked = repository.persist_pdf(source.to_str().unwrap(), &link_settings).unwrap();
        repository.reconcile_document_storage(&linked.document_id, &copy_settings).unwrap();
        let copied = repository.load_document_library().unwrap().remove(0);
        assert_eq!(copied.storage_mode, "managed-copy");
        assert!(PathBuf::from(&copied.path).is_file());

        repository.reconcile_document_storage(&linked.document_id, &link_settings).unwrap();
        let stable = repository.load_document_library().unwrap().remove(0);
        assert_eq!(stable.storage_mode, "managed-copy");
        assert_eq!(stable.path, copied.path);

        fs::remove_file(&stable.path).unwrap();
        repository.reconcile_document_storage(&linked.document_id, &link_settings).unwrap();
        let fallback = repository.load_document_library().unwrap().remove(0);
        assert_eq!(fallback.storage_mode, "linked-file");
        assert_eq!(fallback.path, path_to_user_string(&fs::canonicalize(&source).unwrap()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn markdown_storage_mode_upgrades_to_copy_and_preserves_the_copy_path() {
        let root = test_root("markdown-storage-mode-transition");
        let source = root.join("source").join("notes.md");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "notes").unwrap();
        let repository = LocalStorageRepository::new(AppPaths::new(root.clone()));
        repository.initialize().unwrap();
        let link_settings = AppSettings { markdown_import_mode: DocumentImportMode::Link, ..AppSettings::default() };
        let copy_settings = AppSettings::default();

        let linked = repository.persist_markdown(source.to_str().unwrap(), &link_settings).unwrap();
        let copied = repository.persist_markdown(source.to_str().unwrap(), &copy_settings).unwrap();
        assert_eq!(copied.document_id, linked.document_id);
        assert_eq!(copied.storage_mode, "managed-copy");
        assert!(PathBuf::from(&copied.storage_path).is_file());
        let stable = repository.persist_markdown(source.to_str().unwrap(), &link_settings).unwrap();
        assert_eq!(stable.storage_mode, "managed-copy");
        assert_eq!(stable.storage_path, copied.storage_path);

        let library = repository.load_document_library().unwrap();
        assert_eq!(library.len(), 1);
        assert_eq!(library[0].storage_mode, "managed-copy");
        assert_eq!(library[0].path, stable.storage_path);
        let metadata: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(PathBuf::from(&stable.data_path).join("metadata.json")).unwrap(),
        ).unwrap();
        assert_eq!(metadata["storageMode"], "managed-copy");
        assert_eq!(metadata["documentPath"], stable.storage_path);
        fs::remove_dir_all(root).unwrap();
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bambook-storage-{name}-{}-{}",
            std::process::id(),
            super::next_probe_id()
        ))
    }
}
