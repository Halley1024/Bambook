pub(crate) fn sanitize_filename(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            ch if ch.is_control() => '-',
            _ => ch,
        })
        .collect();

    let sanitized = sanitized.trim().trim_end_matches([' ', '.']);
    let stem = sanitized.split('.').next().unwrap_or_default();
    let is_reserved = matches!(
        stem.to_ascii_uppercase().as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    );

    match (sanitized.is_empty(), is_reserved) {
        (true, _) => "document".to_string(),
        (false, true) => format!("_{sanitized}"),
        (false, false) => sanitized.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn creates_windows_safe_file_names() {
        assert_eq!(sanitize_filename("report:2026.pdf"), "report-2026.pdf");
        assert_eq!(sanitize_filename("CON"), "_CON");
        assert_eq!(sanitize_filename("nul.txt"), "_nul.txt");
        assert_eq!(sanitize_filename("title.  "), "title");
        assert_eq!(sanitize_filename("..."), "document");
    }
}
