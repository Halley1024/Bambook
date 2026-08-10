use super::NormalizedText;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SearchRange {
    pub(crate) normalized_start: usize,
    pub(crate) normalized_end: usize,
    pub(crate) first_unit: usize,
    pub(crate) last_unit: usize,
}

pub(crate) fn find_matches(
    haystack: &NormalizedText,
    needle: &str,
    whole_word: bool,
) -> Vec<SearchRange> {
    if needle.is_empty() {
        return Vec::new();
    }
    let mut results = Vec::new();
    let mut cursor = 0;
    while cursor <= haystack.value.len() {
        let Some(relative) = haystack.value[cursor..].find(needle) else {
            break;
        };
        let start = cursor + relative;
        let end = start + needle.len();
        if !whole_word || is_word_boundary(&haystack.value, start, end) {
            if let (Some(first), Some(last)) =
                (unit_at(haystack, start), unit_before(haystack, end))
            {
                results.push(SearchRange {
                    normalized_start: start,
                    normalized_end: end,
                    first_unit: first,
                    last_unit: last,
                });
            }
        }
        cursor = next_char_boundary(&haystack.value, start);
    }
    results
}

fn is_word_boundary(value: &str, start: usize, end: usize) -> bool {
    let before = value[..start].chars().next_back();
    let after = value[end..].chars().next();
    !before.is_some_and(is_word_character) && !after.is_some_and(is_word_character)
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn unit_at(text: &NormalizedText, offset: usize) -> Option<usize> {
    text.units
        .iter()
        .position(|unit| unit.normalized_start <= offset && unit.normalized_end > offset)
}

fn unit_before(text: &NormalizedText, offset: usize) -> Option<usize> {
    text.units
        .iter()
        .rposition(|unit| unit.normalized_start < offset && unit.normalized_end <= offset)
}

fn next_char_boundary(value: &str, offset: usize) -> usize {
    value[offset..]
        .chars()
        .next()
        .map_or(value.len() + 1, |character| offset + character.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::search::{normalize_symbols, SourceSymbol};

    fn normalize(value: &str) -> NormalizedText {
        let symbols = value
            .char_indices()
            .map(|(start, character)| SourceSymbol {
                text: character.to_string(),
                source_start: start,
                source_end: start + character.len_utf8(),
                line: 1,
                rect: None,
            })
            .collect::<Vec<_>>();
        normalize_symbols(&symbols, false, false)
    }

    #[test]
    fn finds_overlapping_unicode_safe_matches() {
        assert_eq!(find_matches(&normalize("aaaa"), "aa", false).len(), 3);
        assert_eq!(find_matches(&normalize("中文中文"), "中文", false).len(), 2);
    }

    #[test]
    fn supports_whole_words() {
        assert_eq!(
            find_matches(&normalize("cat scatter cat"), "cat", true).len(),
            2
        );
    }
}
