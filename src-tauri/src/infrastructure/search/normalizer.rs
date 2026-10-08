use unicode_normalization::UnicodeNormalization;

use crate::domain::search::SearchRect;

#[derive(Debug, Clone)]
pub(crate) struct SourceSymbol {
    pub(crate) text: String,
    pub(crate) source_start: usize,
    pub(crate) source_end: usize,
    pub(crate) line: usize,
    pub(crate) rect: Option<SearchRect>,
}

#[derive(Debug, Clone)]
pub(crate) struct NormalizedUnit {
    pub(crate) normalized_start: usize,
    pub(crate) normalized_end: usize,
    pub(crate) source_start: usize,
    pub(crate) source_end: usize,
    pub(crate) line: usize,
    pub(crate) rect: Option<SearchRect>,
}

#[derive(Debug, Clone)]
pub(crate) struct NormalizedText {
    pub(crate) value: String,
    pub(crate) units: Vec<NormalizedUnit>,
}

pub(crate) fn normalize_symbols(
    symbols: &[SourceSymbol],
    case_sensitive: bool,
    join_line_hyphens: bool,
) -> NormalizedText {
    let mut value = String::new();
    let mut units = Vec::new();
    let mut index = 0;
    while index < symbols.len() {
        let symbol = &symbols[index];
        if symbol.text == "\u{00ad}" {
            index += 1;
            continue;
        }
        if join_line_hyphens
            && symbol.text == "-"
            && symbols.get(index + 1).is_some_and(|next| next.text == "\n")
            && previous_is_alphanumeric(symbols, index)
            && next_is_alphanumeric(symbols, index + 2)
        {
            index += 2;
            continue;
        }

        let normalized_piece: String = symbol.text.nfkc().collect();
        let normalized_piece = if case_sensitive {
            normalized_piece
        } else {
            normalized_piece.to_lowercase()
        };
        for character in normalized_piece.chars() {
            let character = if character.is_whitespace() {
                ' '
            } else {
                character
            };
            if character == ' ' && value.ends_with(' ') {
                continue;
            }
            let normalized_start = value.len();
            value.push(character);
            units.push(NormalizedUnit {
                normalized_start,
                normalized_end: value.len(),
                source_start: symbol.source_start,
                source_end: symbol.source_end,
                line: symbol.line,
                rect: symbol.rect,
            });
        }
        index += 1;
    }
    NormalizedText { value, units }
}

fn previous_is_alphanumeric(symbols: &[SourceSymbol], index: usize) -> bool {
    symbols[..index]
        .iter()
        .rev()
        .flat_map(|symbol| symbol.text.chars().rev())
        .find(|character| !character.is_whitespace())
        .is_some_and(char::is_alphanumeric)
}

fn next_is_alphanumeric(symbols: &[SourceSymbol], index: usize) -> bool {
    symbols[index..]
        .iter()
        .flat_map(|symbol| symbol.text.chars())
        .find(|character| !character.is_whitespace())
        .is_some_and(char::is_alphanumeric)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbols(value: &str) -> Vec<SourceSymbol> {
        value
            .char_indices()
            .map(|(start, character)| SourceSymbol {
                text: character.to_string(),
                source_start: start,
                source_end: start + character.len_utf8(),
                line: 1,
                rect: None,
            })
            .collect()
    }

    #[test]
    fn normalizes_full_width_case_and_whitespace_with_mapping() {
        let normalized = normalize_symbols(&symbols("Ａ  B\n中"), false, false);
        assert_eq!(normalized.value, "a b 中");
        assert_eq!(normalized.units[0].source_start, 0);
        assert_eq!(
            normalized.units.last().unwrap().source_end,
            "Ａ  B\n中".len()
        );
    }

    #[test]
    fn joins_pdf_line_end_hyphenation() {
        let normalized = normalize_symbols(&symbols("inter-\nnational"), false, true);
        assert_eq!(normalized.value, "international");
    }

    #[test]
    fn preserves_mapping_when_nfkc_expands_special_characters() {
        let source = "ﬁle①";
        let normalized = normalize_symbols(&symbols(source), false, false);
        assert_eq!(normalized.value, "file1");
        assert_eq!(normalized.units[0].source_start, 0);
        assert_eq!(normalized.units[1].source_start, 0);
        assert_eq!(normalized.units.last().unwrap().source_end, source.len());
    }
}
