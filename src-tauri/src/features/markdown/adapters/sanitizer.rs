pub(super) fn sanitize_link(value: &str, image: bool) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || is_relative(trimmed) {
        return trimmed.to_string();
    }

    let scheme = trimmed
        .split_once(':')
        .map(|(scheme, _)| scheme.to_ascii_lowercase());
    let allowed = match scheme.as_deref() {
        Some("http" | "https") => true,
        Some("mailto") => !image,
        _ => false,
    };
    if allowed {
        trimmed.to_string()
    } else {
        String::new()
    }
}

fn is_relative(value: &str) -> bool {
    !value.contains(':') && !value.starts_with("//")
}

#[cfg(test)]
mod tests {
    use super::sanitize_link;

    #[test]
    fn rejects_executable_schemes() {
        assert_eq!(sanitize_link("javascript:alert(1)", false), "");
        assert_eq!(sanitize_link("data:text/html,test", true), "");
        assert_eq!(
            sanitize_link("https://example.com", false),
            "https://example.com"
        );
        assert_eq!(sanitize_link("./image.png", true), "./image.png");
    }
}
