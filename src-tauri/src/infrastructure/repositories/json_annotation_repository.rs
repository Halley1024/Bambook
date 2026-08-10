use std::{fs, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use serde::{Deserialize, Serialize};

use crate::{
    application::ports::AnnotationRepository,
    domain::{
        annotation::{Annotation, AnnotationRect, AreaAnnotationStyle},
        settings::AppSettings,
    },
    error::AppResult,
    infrastructure::{
        filesystem::write_pretty_json,
        paths::AppPaths,
    },
};

pub(crate) struct JsonAnnotationRepository {
    paths: AppPaths,
}

impl JsonAnnotationRepository {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
}

impl AnnotationRepository for JsonAnnotationRepository {
    fn load(&self, document_id: &str, source_path: Option<&str>, settings: &AppSettings) -> AppResult<Vec<Annotation>> {
        let managed = self.paths.annotation_file(document_id, settings)?;
        let sidecar = source_path.and_then(|path| std::path::Path::new(path).parent()).map(|parent| parent.join("annotations.json"));
        let legacy = self.paths.legacy_annotation_file(document_id)?;
        let mut selected: Option<AnnotationCandidate> = None;
        for path in [sidecar, Some(managed), Some(legacy)].into_iter().flatten() {
            let Some(candidate) = read_annotation_candidate(&path) else { continue };
            if selected.as_ref().is_none_or(|current| candidate.updated_at > current.updated_at) {
                selected = Some(candidate);
            }
        }
        Ok(selected.map(|candidate| candidate.annotations).unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn save(
        &self,
        document_id: &str,
        source_path: Option<&str>,
        annotations: &[Annotation],
        settings: &AppSettings,
    ) -> AppResult<()> {
        let path = self.paths.annotation_file(document_id, settings)?;
        let stored: Vec<StoredAnnotation> = annotations.iter().map(Into::into).collect();
        write_annotation_file(&path, &stored)?;
        if let Some(sidecar) = source_path.and_then(|value| std::path::Path::new(value).parent()).map(|parent| parent.join("annotations.json")) {
            if sidecar.is_file() && sidecar != path { write_annotation_file(&sidecar, &stored)?; }
        }
        Ok(())
    }

    fn migrate(
        &self,
        from_document_id: &str,
        to_document_id: &str,
        settings: &AppSettings,
    ) -> AppResult<()> {
        let from_path = self.paths.annotation_file(from_document_id, settings)?;
        let to_path = self.paths.annotation_file(to_document_id, settings)?;
        if from_path == to_path {
            return Ok(());
        }
        if !from_path.exists() {
            return Ok(());
        }
        if let Some(parent) = to_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&from_path, &to_path)?;
        Ok(())
    }
}

const ANNOTATION_SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredAnnotationDocument {
    schema_version: u32,
    updated_at: u64,
    annotations: Vec<StoredAnnotation>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StoredAnnotationPayload {
    Versioned(StoredAnnotationDocument),
    Legacy(Vec<StoredAnnotation>),
}

struct AnnotationCandidate {
    updated_at: u64,
    annotations: Vec<StoredAnnotation>,
}

fn read_annotation_candidate(path: &Path) -> Option<AnnotationCandidate> {
    read_annotation_path(path).or_else(|| read_annotation_path(&backup_path(path)))
}

fn read_annotation_path(path: &Path) -> Option<AnnotationCandidate> {
    let content = fs::read_to_string(path).ok()?;
    let payload: StoredAnnotationPayload = serde_json::from_str(&content).ok()?;
    match payload {
        StoredAnnotationPayload::Versioned(document) if document.schema_version <= ANNOTATION_SCHEMA_VERSION => Some(AnnotationCandidate {
            updated_at: document.updated_at,
            annotations: document.annotations,
        }),
        StoredAnnotationPayload::Versioned(_) => None,
        StoredAnnotationPayload::Legacy(annotations) => Some(AnnotationCandidate {
            updated_at: file_modified_millis(path),
            annotations,
        }),
    }
}

fn write_annotation_file(path: &Path, annotations: &[StoredAnnotation]) -> AppResult<()> {
    if read_annotation_path(path).is_some() {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        fs::copy(path, backup_path(path))?;
    } else if path.is_file() {
        fs::copy(path, corrupt_path(path))?;
    }
    write_pretty_json(path, &StoredAnnotationDocument {
        schema_version: ANNOTATION_SCHEMA_VERSION,
        updated_at: now_millis(),
        annotations: annotations.to_vec(),
    })
}

fn corrupt_path(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("annotations.json");
    path.with_file_name(format!("{name}.corrupt-{}", now_millis()))
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("annotations.json");
    path.with_file_name(format!("{name}.bak"))
}

fn file_modified_millis(path: &Path) -> u64 {
    fs::metadata(path).and_then(|metadata| metadata.modified()).ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .and_then(|duration| duration.as_millis().try_into().ok()).unwrap_or_default()
}

fn now_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().try_into().unwrap_or(u64::MAX)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredAnnotation {
    id: String,
    document_id: String,
    document_title: String,
    #[serde(rename = "type")]
    annotation_type: String,
    page: Option<u32>,
    selected_text: String,
    note: String,
    #[serde(default, skip_serializing_if = "is_false")]
    has_note: bool,
    color: String,
    created_at: String,
    updated_at: String,
    rects: Option<Vec<StoredAnnotationRect>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    area_style: Option<StoredAreaAnnotationStyle>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredAreaAnnotationStyle {
    background_color: String,
    #[serde(default = "default_area_opacity")]
    opacity: f64,
    border_style: String,
    border_width: f64,
    radius: f64,
    rounded_corners: StoredRoundedCorners,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredRoundedCorners {
    top_left: bool,
    top_right: bool,
    bottom_right: bool,
    bottom_left: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredAnnotationRect {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl From<StoredAnnotation> for Annotation {
    fn from(value: StoredAnnotation) -> Self {
        Self {
            id: value.id,
            document_id: value.document_id,
            document_title: value.document_title,
            annotation_type: value.annotation_type,
            page: value.page,
            selected_text: value.selected_text,
            note: value.note,
            has_note: value.has_note,
            color: value.color,
            created_at: value.created_at,
            updated_at: value.updated_at,
            rects: value
                .rects
                .map(|rects| rects.into_iter().map(Into::into).collect()),
            area_style: value.area_style.map(Into::into),
        }
    }
}

impl From<&Annotation> for StoredAnnotation {
    fn from(value: &Annotation) -> Self {
        Self {
            id: value.id.clone(),
            document_id: value.document_id.clone(),
            document_title: value.document_title.clone(),
            annotation_type: value.annotation_type.clone(),
            page: value.page,
            selected_text: value.selected_text.clone(),
            note: value.note.clone(),
            has_note: value.has_note,
            color: value.color.clone(),
            created_at: value.created_at.clone(),
            updated_at: value.updated_at.clone(),
            rects: value
                .rects
                .as_ref()
                .map(|rects| rects.iter().map(Into::into).collect()),
            area_style: value.area_style.as_ref().map(Into::into),
        }
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

fn default_area_opacity() -> f64 { 0.22 }

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};
    use crate::{application::ports::AnnotationRepository, domain::{annotation::Annotation, settings::AppSettings}, infrastructure::paths::AppPaths};
    use super::{backup_path, JsonAnnotationRepository, StoredAnnotation, StoredAnnotationDocument};

    #[test]
    fn chooses_the_newer_annotation_file_between_sidecar_and_managed_storage() {
        let root = test_root("newest");
        let source = root.join("source").join("book.pdf");
        fs::create_dir_all(source.parent().unwrap()).unwrap(); fs::write(&source, b"pdf").unwrap();
        let managed = root.join("storage").join("pdf-test").join("annotations.json");
        write_document(source.parent().unwrap().join("annotations.json"), 10, "sidecar");
        write_document(managed.clone(), 20, "managed");
        let repository = JsonAnnotationRepository::new(AppPaths::new(root.clone()));
        assert_eq!(repository.load("pdf-test", source.to_str(), &AppSettings::default()).unwrap()[0].selected_text, "managed");
        write_document(source.parent().unwrap().join("annotations.json"), 30, "sidecar-new");
        assert_eq!(repository.load("pdf-test", source.to_str(), &AppSettings::default()).unwrap()[0].selected_text, "sidecar-new");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovers_from_backup_and_migrates_legacy_arrays_on_save() {
        let root = test_root("recovery");
        let path = root.join("storage").join("pdf-test").join("annotations.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "not json").unwrap();
        write_document(backup_path(&path), 12, "backup");
        let repository = JsonAnnotationRepository::new(AppPaths::new(root.clone()));
        let recovered = repository.load("pdf-test", None, &AppSettings::default()).unwrap();
        assert_eq!(recovered[0].selected_text, "backup");
        repository.save("pdf-test", None, &recovered, &AppSettings::default()).unwrap();
        assert!(fs::read_dir(path.parent().unwrap()).unwrap().flatten()
            .any(|entry| entry.file_name().to_string_lossy().starts_with("annotations.json.corrupt-")));

        fs::write(&path, serde_json::to_vec_pretty(&vec![StoredAnnotation::from(&annotation("legacy"))]).unwrap()).unwrap();
        let legacy = repository.load("pdf-test", None, &AppSettings::default()).unwrap();
        repository.save("pdf-test", None, &legacy, &AppSettings::default()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(json["schemaVersion"], 4);
        assert!(backup_path(&path).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    fn write_document(path: PathBuf, updated_at: u64, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let document = StoredAnnotationDocument { schema_version: 4, updated_at, annotations: vec![StoredAnnotation::from(&annotation(text))] };
        fs::write(path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
    }

    fn annotation(text: &str) -> Annotation {
        Annotation { id: format!("a-{text}"), document_id: "pdf-test".into(), document_title: "Book".into(), annotation_type: "highlight".into(),
            page: Some(1), selected_text: text.into(), note: String::new(), has_note: false, color: "#ffff00".into(),
            created_at: "now".into(), updated_at: "now".into(), rects: None, area_style: None }
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bambook-annotations-{name}-{}", std::process::id()))
    }
}

impl From<StoredAnnotationRect> for AnnotationRect {
    fn from(value: StoredAnnotationRect) -> Self {
        Self {
            left: value.left,
            top: value.top,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<&AnnotationRect> for StoredAnnotationRect {
    fn from(value: &AnnotationRect) -> Self {
        Self {
            left: value.left,
            top: value.top,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<StoredAreaAnnotationStyle> for AreaAnnotationStyle {
    fn from(value: StoredAreaAnnotationStyle) -> Self {
        Self { background_color: value.background_color, opacity: value.opacity, border_style: value.border_style,
            border_width: value.border_width, radius: value.radius,
            top_left: value.rounded_corners.top_left, top_right: value.rounded_corners.top_right,
            bottom_right: value.rounded_corners.bottom_right, bottom_left: value.rounded_corners.bottom_left }
    }
}

impl From<&AreaAnnotationStyle> for StoredAreaAnnotationStyle {
    fn from(value: &AreaAnnotationStyle) -> Self {
        Self { background_color: value.background_color.clone(), opacity: value.opacity, border_style: value.border_style.clone(),
            border_width: value.border_width, radius: value.radius,
            rounded_corners: StoredRoundedCorners { top_left: value.top_left, top_right: value.top_right,
                bottom_right: value.bottom_right, bottom_left: value.bottom_left } }
    }
}
