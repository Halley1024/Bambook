use std::fs;

use crate::{
    application::ports::AnnotationExporter,
    domain::{annotation::Annotation, document::ReaderDocument, settings::AppSettings},
    error::AppResult,
    infrastructure::{paths::AppPaths, utils::sanitize_filename},
};

pub(crate) struct MarkdownAnnotationExporter {
    paths: AppPaths,
}

impl MarkdownAnnotationExporter {
    pub(crate) fn new(paths: AppPaths) -> Self {
        Self { paths }
    }
}

impl AnnotationExporter for MarkdownAnnotationExporter {
    fn export_markdown(
        &self,
        document: &ReaderDocument,
        annotations: &[Annotation],
        settings: &AppSettings,
    ) -> AppResult<String> {
        let directory = if document.kind == "pdf" {
            self.paths.indexed_document_directory(&document.id)
                .unwrap_or_else(|| self.paths.pdf_directory_for(&document.id, settings))
        } else {
            self.paths.export_dir(settings)
        };
        fs::create_dir_all(&directory)?;

        let safe_title = sanitize_filename(&document.title);
        let path = directory.join(format!("{safe_title}-annotations.md"));
        let output = render_markdown(document, annotations);
        fs::write(&path, output)?;
        Ok(path.to_string_lossy().to_string())
    }
}

fn render_markdown(document: &ReaderDocument, annotations: &[Annotation]) -> String {
    let mut output = format!("# {}\n\n来源：`{}`\n\n", document.title, document.path);

    if annotations.is_empty() {
        output.push_str("_暂无批注。_\n");
        return output;
    }

    for item in annotations {
        if let Some(page) = item.page {
            output.push_str(&format!("## 第 {page} 页\n\n"));
        } else {
            output.push_str("## Markdown 摘录\n\n");
        }
        output.push_str(&format!(
            "> {}\n\n",
            item.selected_text.replace('\n', "\n> ")
        ));
        if !item.note.trim().is_empty() {
            output.push_str(&format!("{}\n\n", item.note.trim()));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use crate::domain::{annotation::Annotation, document::ReaderDocument};

    use super::render_markdown;

    #[test]
    fn renders_empty_and_populated_exports() {
        let document = ReaderDocument {
            id: "doc".into(),
            title: "Guide".into(),
            path: "C:\\docs\\guide.pdf".into(),
            kind: "pdf".into(),
        };
        assert!(render_markdown(&document, &[]).contains("暂无批注"));

        let annotation = Annotation {
            id: "a1".into(),
            document_id: "doc".into(),
            document_title: "Guide".into(),
            annotation_type: "underline".into(),
            page: Some(2),
            selected_text: "first\nsecond".into(),
            note: "remember".into(),
            has_note: true,
            color: "#000".into(),
            created_at: "now".into(),
            updated_at: "now".into(),
            rects: None,
            area_style: None,
        };
        let output = render_markdown(&document, &[annotation]);
        assert!(output.contains("## 第 2 页"));
        assert!(output.contains("> first\n> second"));
        assert!(output.contains("remember"));
    }
}
