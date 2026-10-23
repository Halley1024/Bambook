use crate::features::pdf::JsonPdfDataRepository;
use crate::features::pdf::PdfBookmark;
use crate::features::pdf::PdfDataRepository;
use crate::features::pdf::PdfReadingState;
use std::sync::Arc;
use std::{fs, path::PathBuf};

use crate::features::library::LibraryRepository;
use crate::features::settings::AppSettings;
use crate::features::settings::DocumentImportMode;
use crate::shared::error::AppError;

use super::{path_to_user_string, LibraryPaths, LocalLibraryRepository};

#[test]
fn initializes_every_required_directory() {
    let root = test_root("initialize");
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    let pdf_data = JsonPdfDataRepository::new(repository.clone());
    repository.initialize().unwrap();

    let staged = repository
        .prepare_pdf(source.to_str().unwrap(), &AppSettings::default())
        .unwrap();
    let document_directory = root.join("storage").join(&staged.document_id);
    assert_eq!(staged.display_name, "guide.pdf");
    assert!(!document_directory.exists());
    assert!(repository.load_document_library().unwrap().is_empty());
    pdf_data
        .save_pdf_reading_state(
            &staged.document_id,
            None,
            &PdfReadingState {
                page: 1,
                page_y: None,
                zoom_mode: "fit-width".into(),
                scale: 1.0,
                updated_at: "now".into(),
            },
        )
        .unwrap();
    assert!(
        !document_directory.exists(),
        "background reading-state save must not create storage"
    );

    let managed = repository
        .persist_pdf(source.to_str().unwrap(), &AppSettings::default())
        .unwrap();
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
    pdf_data
        .save_pdf_bookmarks(&managed.document_id, None, std::slice::from_ref(&bookmark))
        .unwrap();
    assert_eq!(
        pdf_data
            .load_pdf_bookmarks(&managed.document_id, None)
            .unwrap(),
        vec![bookmark.clone()]
    );
    fs::write(source_directory.join("bookmarks.json"), format!(
            "[{{\"id\":\"local\",\"documentId\":\"{}\",\"page\":7,\"x\":0.2,\"y\":0.3,\"title\":\"Local sidecar\",\"createdAt\":\"now\"}}]",
            managed.document_id
        )).unwrap();
    let local_bookmarks = pdf_data
        .load_pdf_bookmarks(&managed.document_id, source.to_str())
        .unwrap();
    assert_eq!(local_bookmarks[0].title, "Local sidecar");
    fs::remove_file(source_directory.join("bookmarks.json")).unwrap();
    assert_eq!(
        pdf_data
            .load_pdf_bookmarks(&managed.document_id, source.to_str())
            .unwrap(),
        vec![bookmark]
    );

    let reading_state = PdfReadingState {
        page: 2,
        page_y: Some(0.4),
        zoom_mode: "fit-width".into(),
        scale: 1.25,
        updated_at: "now".into(),
    };
    pdf_data
        .save_pdf_reading_state(&managed.document_id, None, &reading_state)
        .unwrap();
    assert_eq!(
        pdf_data
            .load_pdf_reading_state(&managed.document_id, None)
            .unwrap(),
        Some(reading_state)
    );
    let export_root = root.join("exports");
    fs::create_dir_all(&export_root).unwrap();
    let package = repository
        .export_document_package(&managed.document_id, export_root.to_str().unwrap())
        .unwrap();
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
    let packaged = library
        .iter()
        .find(|entry| entry.id == format!("{}-package", managed.document_id))
        .unwrap();
    assert_eq!(packaged.storage_mode, "linked-file");
    assert_eq!(packaged.data_path, path_to_user_string(&package));
    assert_eq!(
        packaged.path,
        path_to_user_string(&package.join("guide.pdf"))
    );
    assert!(!root.join("storage").join(&packaged.id).exists());
    let package_metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(package.join("metadata.json")).unwrap()).unwrap();
    assert_eq!(package_metadata["documentId"], packaged.id);
    assert_eq!(package_metadata["storageMode"], "linked-file");
    let second_package = PathBuf::from(
        repository
            .export_document_package(&managed.document_id, export_root.to_str().unwrap())
            .unwrap(),
    );
    assert_ne!(second_package, package);
    let library = repository.load_document_library().unwrap();
    assert_eq!(library.len(), 2);
    let packaged = library
        .iter()
        .find(|entry| entry.id == format!("{}-package", managed.document_id))
        .unwrap();
    assert_eq!(packaged.data_path, path_to_user_string(&second_package));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn link_mode_keeps_the_source_in_place_and_creates_companion_metadata() {
    let root = test_root("linked-pdf");
    let source = root.join("incoming").join("linked.pdf");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"pdf fixture").unwrap();
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    let settings = AppSettings {
        pdf_import_mode: DocumentImportMode::Link,
        ..AppSettings::default()
    };

    let staged = repository
        .prepare_pdf(source.to_str().unwrap(), &settings)
        .unwrap();
    let data_directory = root.join("storage").join(&staged.document_id);
    assert!(!data_directory.exists());
    assert!(repository.load_document_library().unwrap().is_empty());
    let managed = repository
        .persist_pdf(source.to_str().unwrap(), &settings)
        .unwrap();
    assert_eq!(
        managed.storage_path,
        path_to_user_string(&fs::canonicalize(&source).unwrap())
    );
    assert!(!data_directory.join("document.pdf").exists());
    assert!(data_directory.join("metadata.json").is_file());
    assert_eq!(
        repository.load_document_library().unwrap()[0].storage_mode,
        "linked-file"
    );
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();

    let managed = repository
        .persist_pdf(source.to_str().unwrap(), &AppSettings::default())
        .unwrap();
    let original_storage_path = managed.storage_path;
    let updated = repository
        .relink_document(&managed.document_id, relocated.to_str().unwrap())
        .unwrap();

    assert_eq!(updated.storage_mode, "managed-copy");
    assert_eq!(updated.path, original_storage_path);
    assert_eq!(
        updated.source_path,
        path_to_user_string(&fs::canonicalize(&relocated).unwrap())
    );
    assert!(PathBuf::from(&updated.path).is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn markdown_storage_is_created_only_when_the_document_is_saved() {
    let root = test_root("deferred-markdown");
    let source = root.join("incoming").join("notes.md");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "# Notes").unwrap();
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    let settings = AppSettings::default();

    let staged = repository
        .prepare_markdown(source.to_str().unwrap(), &settings)
        .unwrap();
    let data_directory = root.join("storage").join(&staged.document_id);
    assert!(!data_directory.exists());
    assert!(repository.load_document_library().unwrap().is_empty());

    repository
        .persist_markdown(source.to_str().unwrap(), &settings)
        .unwrap();
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    let settings = AppSettings {
        pdf_import_mode: DocumentImportMode::Link,
        ..AppSettings::default()
    };

    let first = repository
        .persist_pdf(first_source.to_str().unwrap(), &settings)
        .unwrap();
    let staged = repository
        .prepare_pdf(relocated_source.to_str().unwrap(), &settings)
        .unwrap();
    assert_eq!(staged.document_id, first.document_id);
    let updated = repository
        .persist_pdf(relocated_source.to_str().unwrap(), &settings)
        .unwrap();
    assert_eq!(updated.document_id, first.document_id);

    let library = repository.load_document_library().unwrap();
    assert_eq!(library.len(), 1);
    let relocated = path_to_user_string(&fs::canonicalize(&relocated_source).unwrap());
    assert_eq!(library[0].source_path, relocated);
    assert_eq!(library[0].path, relocated);
    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(PathBuf::from(&first.data_path).join("metadata.json")).unwrap(),
    )
    .unwrap();
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    repository
        .persist_pdf(first_source.to_str().unwrap(), &AppSettings::default())
        .unwrap();

    let error = repository
        .persist_pdf(duplicate_source.to_str().unwrap(), &AppSettings::default())
        .unwrap_err();
    assert!(
        matches!(error, AppError::DuplicateManagedDocumentName { name } if name == "guide.pdf")
    );
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    let settings = AppSettings {
        markdown_import_mode: DocumentImportMode::Link,
        ..AppSettings::default()
    };

    let first = repository
        .persist_markdown(first_source.to_str().unwrap(), &settings)
        .unwrap();
    let staged = repository
        .prepare_markdown(relocated_source.to_str().unwrap(), &settings)
        .unwrap();
    assert_eq!(staged.document_id, first.document_id);
    let updated = repository
        .persist_markdown(relocated_source.to_str().unwrap(), &settings)
        .unwrap();
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    repository
        .persist_markdown(first_source.to_str().unwrap(), &AppSettings::default())
        .unwrap();

    let error = repository
        .persist_markdown(duplicate_source.to_str().unwrap(), &AppSettings::default())
        .unwrap_err();
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    let link_settings = AppSettings {
        pdf_import_mode: DocumentImportMode::Link,
        ..AppSettings::default()
    };
    let copy_settings = AppSettings::default();

    let linked = repository
        .persist_pdf(source.to_str().unwrap(), &link_settings)
        .unwrap();
    assert_eq!(linked.storage_mode, "linked-file");
    let copied = repository
        .persist_pdf(source.to_str().unwrap(), &copy_settings)
        .unwrap();
    assert_eq!(copied.document_id, linked.document_id);
    assert_eq!(copied.storage_mode, "managed-copy");
    assert!(PathBuf::from(&copied.storage_path).is_file());

    let stable = repository
        .persist_pdf(source.to_str().unwrap(), &link_settings)
        .unwrap();
    assert_eq!(stable.storage_mode, "managed-copy");
    assert_eq!(stable.storage_path, copied.storage_path);

    fs::remove_file(&stable.storage_path).unwrap();
    let fallback = repository
        .persist_pdf(source.to_str().unwrap(), &link_settings)
        .unwrap();
    assert_eq!(fallback.storage_mode, "linked-file");
    assert_eq!(
        fallback.storage_path,
        path_to_user_string(&fs::canonicalize(&source).unwrap())
    );
    let restored = repository
        .persist_pdf(source.to_str().unwrap(), &copy_settings)
        .unwrap();
    assert_eq!(restored.storage_mode, "managed-copy");
    assert!(PathBuf::from(&restored.storage_path).is_file());

    let library = repository.load_document_library().unwrap();
    assert_eq!(library.len(), 1);
    assert_eq!(library[0].storage_mode, "managed-copy");
    assert_eq!(library[0].path, restored.storage_path);
    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(PathBuf::from(&restored.data_path).join("metadata.json")).unwrap(),
    )
    .unwrap();
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
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    let link_settings = AppSettings {
        pdf_import_mode: DocumentImportMode::Link,
        ..AppSettings::default()
    };
    let copy_settings = AppSettings::default();

    let linked = repository
        .persist_pdf(source.to_str().unwrap(), &link_settings)
        .unwrap();
    repository
        .reconcile_document_storage(&linked.document_id, &copy_settings)
        .unwrap();
    let copied = repository.load_document_library().unwrap().remove(0);
    assert_eq!(copied.storage_mode, "managed-copy");
    assert!(PathBuf::from(&copied.path).is_file());

    repository
        .reconcile_document_storage(&linked.document_id, &link_settings)
        .unwrap();
    let stable = repository.load_document_library().unwrap().remove(0);
    assert_eq!(stable.storage_mode, "managed-copy");
    assert_eq!(stable.path, copied.path);

    fs::remove_file(&stable.path).unwrap();
    repository
        .reconcile_document_storage(&linked.document_id, &link_settings)
        .unwrap();
    let fallback = repository.load_document_library().unwrap().remove(0);
    assert_eq!(fallback.storage_mode, "linked-file");
    assert_eq!(
        fallback.path,
        path_to_user_string(&fs::canonicalize(&source).unwrap())
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn markdown_storage_mode_upgrades_to_copy_and_preserves_the_copy_path() {
    let root = test_root("markdown-storage-mode-transition");
    let source = root.join("source").join("notes.md");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "notes").unwrap();
    let repository = Arc::new(LocalLibraryRepository::new(LibraryPaths::new(root.clone())));
    repository.initialize().unwrap();
    let link_settings = AppSettings {
        markdown_import_mode: DocumentImportMode::Link,
        ..AppSettings::default()
    };
    let copy_settings = AppSettings::default();

    let linked = repository
        .persist_markdown(source.to_str().unwrap(), &link_settings)
        .unwrap();
    let copied = repository
        .persist_markdown(source.to_str().unwrap(), &copy_settings)
        .unwrap();
    assert_eq!(copied.document_id, linked.document_id);
    assert_eq!(copied.storage_mode, "managed-copy");
    assert!(PathBuf::from(&copied.storage_path).is_file());
    let stable = repository
        .persist_markdown(source.to_str().unwrap(), &link_settings)
        .unwrap();
    assert_eq!(stable.storage_mode, "managed-copy");
    assert_eq!(stable.storage_path, copied.storage_path);

    let library = repository.load_document_library().unwrap();
    assert_eq!(library.len(), 1);
    assert_eq!(library[0].storage_mode, "managed-copy");
    assert_eq!(library[0].path, stable.storage_path);
    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(PathBuf::from(&stable.data_path).join("metadata.json")).unwrap(),
    )
    .unwrap();
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
