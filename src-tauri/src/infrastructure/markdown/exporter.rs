use std::{fs, path::Path};

use pulldown_cmark::{html, Options, Parser};

use crate::{
    application::ports::MarkdownExporter,
    domain::markdown_ast::MarkdownDocumentModel,
    error::{AppError, AppResult},
    infrastructure::filesystem::write_file_atomically,
};

#[derive(Default)]
pub(crate) struct PulldownMarkdownExporter;

impl MarkdownExporter for PulldownMarkdownExporter {
    fn export_html(&self, document: &MarkdownDocumentModel, output_path: &str) -> AppResult<()> {
        let output_path = validated_output_path(output_path, "html")?;
        let html = render_html(document);
        write_file_atomically(output_path, |temporary_path| {
            fs::write(temporary_path, html.as_bytes())?;
            let generated = fs::read(temporary_path)?;
            if !generated.starts_with(b"<!doctype html>") || !generated.ends_with(b"</html>") {
                return Err(AppError::InvalidExportOutput { format: "HTML" });
            }
            Ok(())
        })
    }

    fn export_pdf(&self, document: &MarkdownDocumentModel, output_path: &str) -> AppResult<()> {
        let output_path = validated_output_path(output_path, "pdf")?;
        let html = render_html(document);
        let mut source =
            mupdf::Document::from_bytes(html.as_bytes(), "text/html").map_err(mupdf_error)?;
        source.layout(595.0, 842.0, 12.0).map_err(mupdf_error)?;
        let pdf = source.convert_to_pdf(0, -1, 0).map_err(mupdf_error)?;
        write_file_atomically(output_path, |temporary_path| {
            let temporary_path = temporary_path
                .to_str()
                .ok_or(AppError::InvalidExportOutput { format: "PDF" })?;
            pdf.save(temporary_path).map_err(mupdf_error)?;
            let generated = mupdf::Document::open(temporary_path).map_err(mupdf_error)?;
            if generated.page_count().map_err(mupdf_error)? <= 0 {
                return Err(AppError::InvalidExportOutput { format: "PDF" });
            }
            Ok(())
        })
    }
}

fn validated_output_path<'a>(output_path: &'a str, expected: &'static str) -> AppResult<&'a Path> {
    let path = Path::new(output_path);
    let valid = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected));
    if !valid {
        return Err(AppError::InvalidExportExtension { expected });
    }
    Ok(path)
}

fn render_html(document: &MarkdownDocumentModel) -> String {
    let mut body = String::new();
    html::push_html(&mut body, Parser::new_ext(&document.source, Options::all()));
    format!(
        "<!doctype html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\"><title>{}</title><style>{}</style></head><body><main>{}</main></body></html>",
        escape_html(&document.title),
        export_styles(),
        body,
    )
}

fn export_styles() -> &'static str {
    "@page{size:A4;margin:18mm}body{color:#20252b;font-family:'Microsoft YaHei','Noto Sans CJK SC',sans-serif;font-size:11pt;line-height:1.7}main{max-width:100%}h1,h2,h3,h4,h5,h6{line-height:1.3;margin:1.25em 0 .55em}h1{font-size:2em}h2{font-size:1.55em}h3{font-size:1.25em}p,ul,ol,blockquote,pre,table{margin:.75em 0}pre{padding:.8em;background:#f4f6f8;overflow-wrap:anywhere}code{font-family:Consolas,'Courier New',monospace}blockquote{margin-left:0;padding-left:1em;border-left:3px solid #cbd5df;color:#4b5563}table{width:100%;border-collapse:collapse}th,td{padding:.45em .6em;border:1px solid #cfd7df;text-align:left}img{max-width:100%;height:auto}a{color:#176749;text-decoration:none}hr{border:0;border-top:1px solid #d8dee5}"
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#39;")
}

fn mupdf_error(error: mupdf::Error) -> AppError {
    AppError::MuPdf(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{render_html, PulldownMarkdownExporter};
    use crate::application::ports::MarkdownExporter;
    use crate::domain::markdown_ast::MarkdownDocumentModel;

    #[test]
    fn wraps_markdown_in_a_printable_html_document() {
        let document = fixture();

        let html = render_html(&document);
        assert!(html.contains("<title>&lt;Notes&gt;</title>"));
        assert!(html.contains("<h1>Heading</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn writes_real_html_and_pdf_files() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let html_path = std::env::temp_dir().join(format!("bambook-export-{suffix}.html"));
        let pdf_path = std::env::temp_dir().join(format!("bambook-export-{suffix}.pdf"));
        let exporter = PulldownMarkdownExporter;
        let document = fixture();
        exporter
            .export_html(&document, html_path.to_str().unwrap())
            .unwrap();
        exporter
            .export_pdf(&document, pdf_path.to_str().unwrap())
            .unwrap();
        let html_bytes = fs::read(&html_path).unwrap();
        let pdf_bytes = fs::read(&pdf_path).unwrap();
        assert!(html_bytes.starts_with(b"<!doctype html>"));
        assert!(pdf_bytes.starts_with(b"%PDF-"));
        let _ = fs::remove_file(html_path);
        let _ = fs::remove_file(pdf_path);
    }

    fn fixture() -> MarkdownDocumentModel {
        MarkdownDocumentModel {
            document_id: "md-test".into(),
            session_id: "test".into(),
            revision: 1,
            path: "notes.md".into(),
            title: "<Notes>".into(),
            source: "# Heading\n\nA **bold** note.".into(),
            nodes: Vec::new(),
            outline: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}
