mod catalog;
mod migration;
mod package;
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

use crate::features::documents::DocumentKind;
use crate::features::library::path_to_user_string;
use crate::features::library::validate_document_id;
use crate::features::library::LibraryPaths;
use crate::features::library::LibraryRepository;
use crate::features::library::ManagedMarkdownDocument;
use crate::features::library::ManagedPdfDocument;
use crate::features::library::StorageLayout;
use crate::features::library::StoredDocumentEntry;
use crate::features::settings::AppSettings;
use crate::features::settings::DocumentImportMode;
use crate::platform::filesystem::atomic_file::write_file_atomically;
use crate::platform::filesystem::json_file::read_optional_json;
use crate::platform::filesystem::json_file::write_pretty_json;
use crate::shared::error::AppError;
use crate::shared::error::AppResult;
use crate::shared::filename::sanitize_filename;

pub(crate) struct LocalLibraryRepository {
    paths: LibraryPaths,
    library_lock: Mutex<()>,
}

impl LocalLibraryRepository {
    pub(crate) fn new(paths: LibraryPaths) -> Self {
        Self {
            paths,
            library_lock: Mutex::new(()),
        }
    }
}

impl LibraryRepository for LocalLibraryRepository {
    fn export_document_package(&self, document_id: &str, destination: &str) -> AppResult<String> {
        self.export_package(document_id, destination)
    }

    fn relink_document(&self, document_id: &str, new_path: &str) -> AppResult<StoredDocumentEntry> {
        self.relink(document_id, new_path)
    }

    fn load_document_library(&self) -> AppResult<Vec<StoredDocumentEntry>> {
        self.load_library()
    }

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
        if let Some(entry) = documents
            .iter()
            .find(|entry| {
                entry.kind == DocumentKind::Pdf
                    && [entry.path.as_str(), entry.source_path.as_str()]
                        .iter()
                        .any(|candidate| {
                            Path::new(candidate).canonicalize().ok().as_ref() == Some(&source)
                        })
            })
            .cloned()
        {
            let open_path = if Path::new(&entry.path).is_file() {
                entry.path.clone()
            } else {
                path_to_user_string(&source)
            };
            return Ok(ManagedPdfDocument {
                document_id: entry.id,
                display_name: entry.title,
                source_path: entry.source_path,
                storage_path: open_path,
                data_path: entry.data_path,
                storage_mode: entry.storage_mode,
            });
        }
        let source_path = path_to_user_string(&source);
        let document_id = format!("pdf-{}", sha256_bytes(normalized_path(&source).as_bytes()));
        let display_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(AppError::MissingFileName)?
            .to_owned();
        if settings.pdf_import_mode == DocumentImportMode::Link {
            if let Some(entry) =
                find_linked_document_by_name(&documents, DocumentKind::Pdf, &display_name)
            {
                return Ok(ManagedPdfDocument {
                    document_id: entry.id.clone(),
                    display_name,
                    source_path: source_path.clone(),
                    storage_path: source_path,
                    data_path: entry.data_path.clone(),
                    storage_mode: entry.storage_mode.clone(),
                });
            }
        }
        let data_path = self.paths.pdf_directory_for(&document_id, settings);
        let storage_mode = if settings.pdf_import_mode == DocumentImportMode::Copy {
            "managed-copy"
        } else {
            "linked-file"
        };
        Ok(ManagedPdfDocument {
            document_id,
            display_name,
            source_path: source_path.clone(),
            storage_path: source_path,
            data_path: path_to_user_string(&data_path),
            storage_mode: storage_mode.into(),
        })
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
            if let Some(existing) =
                find_linked_document_by_name(&documents, DocumentKind::Pdf, &display_name)
                    .filter(|entry| entry.id != document_id)
            {
                let updated = self.relink_document(&existing.id, path)?;
                return Ok(ManagedPdfDocument {
                    document_id: updated.id,
                    display_name: updated.title,
                    source_path: updated.source_path,
                    storage_path: updated.path,
                    data_path: updated.data_path,
                    storage_mode: updated.storage_mode,
                });
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
        let storage_mode = match settings.pdf_import_mode {
            DocumentImportMode::Copy => "managed-copy",
            DocumentImportMode::Link => "linked-file",
        };
        let storage_path = if settings.pdf_import_mode == DocumentImportMode::Copy {
            managed_copy_path.clone()
        } else {
            source.clone()
        };
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

    fn prepare_markdown(
        &self,
        path: &str,
        settings: &AppSettings,
    ) -> AppResult<ManagedMarkdownDocument> {
        let source = fs::canonicalize(path)?;
        let source_path = path_to_user_string(&source);
        let documents = self.read_document_library_unlocked()?;
        if let Some(entry) = documents
            .iter()
            .find(|entry| {
                entry.kind == DocumentKind::Markdown
                    && [entry.path.as_str(), entry.source_path.as_str()]
                        .iter()
                        .any(|candidate| {
                            Path::new(candidate).canonicalize().ok().as_ref() == Some(&source)
                        })
            })
            .cloned()
        {
            let open_path = if Path::new(&entry.path).is_file() {
                entry.path.clone()
            } else {
                source_path.clone()
            };
            return Ok(ManagedMarkdownDocument {
                document_id: entry.id,
                display_name: entry.title,
                source_path: entry.source_path,
                storage_path: open_path,
                data_path: entry.data_path,
                storage_mode: entry.storage_mode,
            });
        }
        let document_id = format!(
            "markdown-{}",
            sha256_bytes(normalized_path(&source).as_bytes())
        );
        let display_name = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(AppError::MissingFileName)?
            .to_owned();
        if settings.markdown_import_mode == DocumentImportMode::Link {
            if let Some(entry) =
                find_linked_document_by_name(&documents, DocumentKind::Markdown, &display_name)
            {
                return Ok(ManagedMarkdownDocument {
                    document_id: entry.id.clone(),
                    display_name,
                    source_path: source_path.clone(),
                    storage_path: source_path,
                    data_path: entry.data_path.clone(),
                    storage_mode: entry.storage_mode.clone(),
                });
            }
        }
        let data_path = self.paths.markdown_directory_for(&document_id, settings);
        let storage_mode = if settings.markdown_import_mode == DocumentImportMode::Copy {
            "managed-copy"
        } else {
            "linked-file"
        };
        Ok(ManagedMarkdownDocument {
            document_id,
            display_name,
            source_path: source_path.clone(),
            storage_path: source_path,
            data_path: path_to_user_string(&data_path),
            storage_mode: storage_mode.into(),
        })
    }

    fn persist_markdown(
        &self,
        path: &str,
        settings: &AppSettings,
    ) -> AppResult<ManagedMarkdownDocument> {
        let source = fs::canonicalize(path)?;
        let source_path = path_to_user_string(&source);
        let documents = self.read_document_library_unlocked()?;
        if let Some(entry) = find_document_by_path(&documents, DocumentKind::Markdown, &source) {
            return self.persist_existing_markdown(entry, &source, settings);
        }
        let document_id = format!(
            "markdown-{}",
            sha256_bytes(normalized_path(&source).as_bytes())
        );
        let display_name = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(AppError::MissingFileName)?
            .to_owned();
        if settings.markdown_import_mode == DocumentImportMode::Link {
            if let Some(existing) =
                find_linked_document_by_name(&documents, DocumentKind::Markdown, &display_name)
                    .filter(|entry| entry.id != document_id)
            {
                let updated = self.relink_document(&existing.id, path)?;
                return Ok(ManagedMarkdownDocument {
                    document_id: updated.id,
                    display_name: updated.title,
                    source_path: updated.source_path,
                    storage_path: updated.path,
                    data_path: updated.data_path,
                    storage_mode: updated.storage_mode,
                });
            }
        }
        if find_document_by_name(&documents, DocumentKind::Markdown, &display_name)
            .is_some_and(|entry| entry.id != document_id)
        {
            return Err(AppError::DuplicateManagedDocumentName { name: display_name });
        }
        let data_path = self.paths.markdown_directory_for(&document_id, settings);
        let copy_path = data_path.join("markdown.md");
        let (storage_path, storage_mode) =
            if settings.markdown_import_mode == DocumentImportMode::Copy {
                write_file_atomically(&copy_path, |temporary| {
                    fs::copy(&source, temporary)?;
                    Ok(())
                })?;
                (copy_path, "managed-copy")
            } else {
                (source.clone(), "linked-file")
            };
        fs::create_dir_all(data_path.join("assets"))?;
        let now = unix_timestamp();
        write_pretty_json(
            &data_path.join("metadata.json"),
            &StoredMarkdownMetadata {
                schema_version: 1,
                document_id: document_id.clone(),
                document_type: "markdown".into(),
                display_name: display_name.clone(),
                source_path: source_path.clone(),
                document_path: path_to_user_string(&storage_path),
                storage_mode: storage_mode.into(),
                created_at: now.clone(),
                updated_at: now,
            },
        )?;
        let managed = ManagedMarkdownDocument {
            document_id,
            display_name,
            source_path,
            storage_path: path_to_user_string(&storage_path),
            data_path: path_to_user_string(&data_path),
            storage_mode: storage_mode.into(),
        };
        self.upsert_markdown_library_entry(&managed)?;
        Ok(managed)
    }

    fn reconcile_document_storage(
        &self,
        document_id: &str,
        settings: &AppSettings,
    ) -> AppResult<()> {
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
}

impl LocalLibraryRepository {
    fn persist_existing_pdf(
        &self,
        entry: &StoredDocumentEntry,
        source: &Path,
        settings: &AppSettings,
    ) -> AppResult<ManagedPdfDocument> {
        let data_path =
            existing_data_path(entry, || self.paths.pdf_directory_for(&entry.id, settings));
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
        let copy_needs_refresh = !copy_available
            || (settings.pdf_import_mode == DocumentImportMode::Copy
                && previous
                    .as_ref()
                    .is_none_or(|metadata| metadata.content_hash != content_hash));
        if use_managed_copy && copy_needs_refresh {
            write_file_atomically(&stable_copy_path, |temporary| {
                fs::copy(source, temporary)?;
                Ok(())
            })?;
        }
        let source_path = path_to_user_string(source);
        let storage_path = if use_managed_copy {
            stable_copy_path
        } else {
            source.to_path_buf()
        };
        let storage_mode = if use_managed_copy {
            "managed-copy"
        } else {
            "linked-file"
        };
        let now = unix_timestamp();
        write_pretty_json(
            &metadata_path,
            &StoredPdfMetadata {
                schema_version: 2,
                document_id: entry.id.clone(),
                document_type: "pdf".into(),
                display_name: entry.title.clone(),
                source_path: source_path.clone(),
                document_path: path_to_user_string(&storage_path),
                storage_mode: storage_mode.into(),
                content_hash,
                created_at: previous
                    .as_ref()
                    .map(|value| value.created_at.clone())
                    .unwrap_or_else(|| now.clone()),
                updated_at: now,
            },
        )?;
        ensure_json_array(&data_path.join("annotations.json"))?;
        ensure_json_array(&data_path.join("bookmarks.json"))?;
        let managed = ManagedPdfDocument {
            document_id: entry.id.clone(),
            display_name: entry.title.clone(),
            source_path,
            storage_path: path_to_user_string(&storage_path),
            data_path: path_to_user_string(&data_path),
            storage_mode: storage_mode.into(),
        };
        self.upsert_pdf_library_entry(&managed)?;
        Ok(managed)
    }

    fn persist_existing_markdown(
        &self,
        entry: &StoredDocumentEntry,
        source: &Path,
        settings: &AppSettings,
    ) -> AppResult<ManagedMarkdownDocument> {
        let data_path = existing_data_path(entry, || {
            self.paths.markdown_directory_for(&entry.id, settings)
        });
        let stable_copy_path = if entry.storage_mode == "managed-copy" && !entry.path.is_empty() {
            PathBuf::from(&entry.path)
        } else {
            data_path.join("markdown.md")
        };
        let source_is_copy =
            entry.storage_mode == "managed-copy" && paths_resolve_to_same_file(&entry.path, source);
        let copy_available = entry.storage_mode == "managed-copy" && stable_copy_path.is_file();
        let use_managed_copy =
            copy_available || settings.markdown_import_mode == DocumentImportMode::Copy;
        if use_managed_copy && !source_is_copy {
            write_file_atomically(&stable_copy_path, |temporary| {
                fs::copy(source, temporary)?;
                Ok(())
            })?;
        }
        fs::create_dir_all(data_path.join("assets"))?;
        let source_path = if source_is_copy {
            entry.source_path.clone()
        } else {
            path_to_user_string(source)
        };
        let storage_path = if use_managed_copy {
            stable_copy_path
        } else {
            source.to_path_buf()
        };
        let storage_mode = if use_managed_copy {
            "managed-copy"
        } else {
            "linked-file"
        };
        let metadata_path = data_path.join("metadata.json");
        let previous: Option<StoredMarkdownMetadata> = read_optional_json(&metadata_path)?;
        let now = unix_timestamp();
        write_pretty_json(
            &metadata_path,
            &StoredMarkdownMetadata {
                schema_version: 1,
                document_id: entry.id.clone(),
                document_type: "markdown".into(),
                display_name: entry.title.clone(),
                source_path: source_path.clone(),
                document_path: path_to_user_string(&storage_path),
                storage_mode: storage_mode.into(),
                created_at: previous
                    .as_ref()
                    .map(|value| value.created_at.clone())
                    .unwrap_or_else(|| now.clone()),
                updated_at: now,
            },
        )?;
        let managed = ManagedMarkdownDocument {
            document_id: entry.id.clone(),
            display_name: entry.title.clone(),
            source_path,
            storage_path: path_to_user_string(&storage_path),
            data_path: path_to_user_string(&data_path),
            storage_mode: storage_mode.into(),
        };
        self.upsert_markdown_library_entry(&managed)?;
        Ok(managed)
    }

    fn managed_pdf_id(&self, path: &Path) -> Option<String> {
        if path.file_name()?.to_string_lossy().to_ascii_lowercase() != "document.pdf" {
            return None;
        }
        let document_id = path.parent()?.file_name()?.to_string_lossy().into_owned();
        validate_document_id(&document_id).ok()?;
        path.parent()?
            .join("metadata.json")
            .is_file()
            .then_some(document_id)
    }

    fn open_managed_pdf(
        &self,
        document_id: &str,
        storage_path: &Path,
    ) -> AppResult<ManagedPdfDocument> {
        validate_document_id(document_id)?;
        let data_path = storage_path.parent().ok_or(AppError::MissingFileName)?;
        let metadata: Option<StoredPdfMetadata> =
            read_optional_json(&data_path.join("metadata.json"))?;
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
            storage_mode: metadata
                .as_ref()
                .map(|value| value.storage_mode.clone())
                .unwrap_or_else(|| "managed-copy".into()),
        })
    }

    fn document_data_directory(&self, document_id: &str) -> AppResult<std::path::PathBuf> {
        validate_document_id(document_id)?;
        Ok(self
            .read_document_library_unlocked()?
            .into_iter()
            .find(|entry| entry.id == document_id)
            .filter(|entry| !entry.data_path.is_empty())
            .map(|entry| std::path::PathBuf::from(entry.data_path))
            .unwrap_or_else(|| self.paths.pdf_directory(document_id)))
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
            data_path: if value.data_path.is_empty() {
                String::new()
            } else {
                normalize_stored_path(value.data_path)
            },
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

fn default_managed_copy() -> String {
    "managed-copy".into()
}

fn find_document_by_name<'a>(
    documents: &'a [StoredDocumentEntry],
    kind: DocumentKind,
    display_name: &str,
) -> Option<&'a StoredDocumentEntry> {
    documents
        .iter()
        .find(|entry| entry.kind == kind && entry.title.eq_ignore_ascii_case(display_name))
}

fn find_document_by_path<'a>(
    documents: &'a [StoredDocumentEntry],
    kind: DocumentKind,
    path: &Path,
) -> Option<&'a StoredDocumentEntry> {
    documents.iter().find(|entry| {
        entry.kind == kind
            && [entry.path.as_str(), entry.source_path.as_str()]
                .iter()
                .any(|candidate| paths_resolve_to_same_file(candidate, path))
    })
}

fn find_linked_document_by_name<'a>(
    documents: &'a [StoredDocumentEntry],
    kind: DocumentKind,
    display_name: &str,
) -> Option<&'a StoredDocumentEntry> {
    documents.iter().find(|entry| {
        entry.kind == kind
            && entry.storage_mode == "linked-file"
            && entry.title.eq_ignore_ascii_case(display_name)
    })
}

fn paths_resolve_to_same_file(candidate: &str, path: &Path) -> bool {
    Path::new(candidate).canonicalize().ok().as_ref() == Some(&path.to_path_buf())
}

fn existing_data_path(entry: &StoredDocumentEntry, fallback: impl FnOnce() -> PathBuf) -> PathBuf {
    if entry.data_path.is_empty() {
        fallback()
    } else {
        PathBuf::from(&entry.data_path)
    }
}

fn normalize_stored_path(value: String) -> String {
    path_to_user_string(Path::new(&value))
}

fn ensure_json_array(path: &Path) -> AppResult<()> {
    if !path.exists() {
        write_pretty_json(path, &Vec::<serde_json::Value>::new())?;
    }
    Ok(())
}

fn existing_sidecar_path(source_path: Option<&str>, file_name: &str) -> Option<std::path::PathBuf> {
    source_path
        .and_then(|path| Path::new(path).parent())
        .map(|parent| parent.join(file_name))
        .filter(|path| path.is_file())
}

fn stored_path_key(path: &str) -> String {
    path.trim()
        .trim_start_matches(r"\\?\")
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
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

impl crate::features::library::DocumentDataLocator for LocalLibraryRepository {
    fn persisted_document_data_directory(
        &self,
        document_id: &str,
    ) -> AppResult<Option<std::path::PathBuf>> {
        validate_document_id(document_id)?;
        let indexed = self
            .read_document_library_unlocked()?
            .into_iter()
            .find(|entry| entry.id == document_id);
        if let Some(entry) = indexed {
            let directory = if entry.data_path.is_empty() {
                self.paths.pdf_directory(document_id)
            } else {
                entry.data_path.into()
            };
            return Ok(Some(directory));
        }
        let legacy = self.paths.pdf_directory(document_id);
        Ok(legacy.is_dir().then_some(legacy))
    }

    fn companion_read_path(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        file_name: &str,
    ) -> AppResult<Option<std::path::PathBuf>> {
        if let Some(sidecar) = existing_sidecar_path(source_path, file_name) {
            return Ok(Some(sidecar));
        }
        let managed = self.document_data_directory(document_id)?.join(file_name);
        Ok(managed.is_file().then_some(managed))
    }
}

#[cfg(test)]
mod tests;
