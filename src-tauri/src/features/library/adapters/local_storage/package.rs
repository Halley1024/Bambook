use super::*;

impl LocalLibraryRepository {
    pub(super) fn export_package(&self, document_id: &str, destination: &str) -> AppResult<String> {
        validate_document_id(document_id)?;
        let entry = self
            .read_document_library_unlocked()?
            .into_iter()
            .find(|entry| entry.id == document_id)
            .ok_or(AppError::UnsupportedDocumentType)?;
        let destination = Path::new(destination);
        if !destination.is_dir() {
            return Err(AppError::MissingParentDirectory);
        }
        let stem = Path::new(&entry.title)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(&entry.title);
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
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "document source is unavailable",
                ),
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
            if data.is_dir() {
                copy_companion_directory(data, &temporary)?;
            }
            fs::copy(source, temporary.join(&package_file_name))?;
            rewrite_package_document_ids(
                &temporary.join("annotations.json"),
                document_id,
                &package_document_id,
            )?;
            rewrite_package_document_ids(
                &temporary.join("bookmarks.json"),
                document_id,
                &package_document_id,
            )?;
            let previous: Option<StoredPdfMetadata> =
                read_optional_json(&temporary.join("metadata.json"))?;
            let now = unix_timestamp();
            write_pretty_json(
                &temporary.join("metadata.json"),
                &StoredPdfMetadata {
                    schema_version: 2,
                    document_id: package_document_id.clone(),
                    document_type: "pdf".into(),
                    display_name: entry.title.clone(),
                    source_path: package_document_path_string.clone(),
                    document_path: package_document_path_string.clone(),
                    storage_mode: "linked-file".into(),
                    content_hash: sha256_file(source)?,
                    created_at: previous
                        .map(|metadata| metadata.created_at)
                        .unwrap_or_else(|| now.clone()),
                    updated_at: now,
                },
            )?;
            fs::rename(&temporary, &package)?;
            Ok(())
        })();
        if export_result.is_err() && temporary.exists() {
            let _ = fs::remove_dir_all(&temporary);
        }
        export_result?;
        let register_result = (|| -> AppResult<()> {
            let _guard = self
                .library_lock
                .lock()
                .map_err(|_| AppError::StateUnavailable)?;
            let mut documents = self.read_document_library_unlocked()?;
            let now = unix_timestamp_millis();
            let created_at = documents
                .iter()
                .find(|item| item.id == package_document_id)
                .map(|item| item.created_at)
                .unwrap_or(now);
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
}

pub(super) fn copy_companion_directory(source: &Path, destination: &Path) -> AppResult<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        let lower_name = name.to_string_lossy().to_ascii_lowercase();
        if matches!(lower_name.as_str(), "document.pdf" | "markdown.md") {
            continue;
        }
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

pub(super) fn rewrite_package_document_ids(
    path: &Path,
    previous_id: &str,
    package_id: &str,
) -> AppResult<()> {
    if !path.is_file() {
        return Ok(());
    }
    let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    rewrite_json_document_ids(&mut value, previous_id, package_id);
    write_pretty_json(path, &value)
}

pub(super) fn rewrite_json_document_ids(
    value: &mut serde_json::Value,
    previous_id: &str,
    package_id: &str,
) {
    match value {
        serde_json::Value::Object(object) => {
            if object.get("documentId").and_then(serde_json::Value::as_str) == Some(previous_id) {
                object.insert(
                    "documentId".into(),
                    serde_json::Value::String(package_id.into()),
                );
            }
            object
                .values_mut()
                .for_each(|value| rewrite_json_document_ids(value, previous_id, package_id));
        }
        serde_json::Value::Array(items) => items
            .iter_mut()
            .for_each(|value| rewrite_json_document_ids(value, previous_id, package_id)),
        _ => {}
    }
}
