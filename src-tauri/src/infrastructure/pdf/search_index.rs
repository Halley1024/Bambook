use mupdf::{Document, Quad, Rect, TextBlockContent, TextPageFlags};

use crate::{
    domain::search::{PdfSearchLocator, SearchLocator, SearchMatch, SearchOptions, SearchRect},
    error::{AppError, AppResult},
    infrastructure::search::{find_matches, normalize_symbols, SourceSymbol},
};

pub(crate) struct PdfPageSearchIndex {
    page_index: u32,
    source: String,
    symbols: Vec<SourceSymbol>,
}

impl PdfPageSearchIndex {
    pub(crate) fn build(document: &Document, page_index: u32) -> AppResult<Self> {
        let page = document.load_page(page_index as i32).map_err(mupdf_error)?;
        let page_bounds = page.bounds().map_err(mupdf_error)?;
        let structured = page
            .to_text_page(TextPageFlags::PRESERVE_WHITESPACE | TextPageFlags::PRESERVE_SPANS)
            .map_err(mupdf_error)?
            .structured();
        let mut source = String::new();
        let mut symbols = Vec::new();
        let mut line_number = 0;
        for block in structured.blocks {
            let TextBlockContent::Text { lines } = block.content else {
                continue;
            };
            for line in lines {
                line_number += 1;
                if line.chars.is_empty() {
                    push_text(
                        &mut source,
                        &mut symbols,
                        &line.text,
                        line_number,
                        Some(normalize_rect(line.bounds, page_bounds)),
                    );
                } else {
                    for character in line.chars {
                        push_text(
                            &mut source,
                            &mut symbols,
                            &character.ch.to_string(),
                            line_number,
                            Some(normalize_rect(quad_bounds(&character.quad), page_bounds)),
                        );
                    }
                }
                push_text(&mut source, &mut symbols, "\n", line_number, None);
            }
        }
        Ok(Self {
            page_index,
            source,
            symbols,
        })
    }

    pub(crate) fn search(
        &self,
        request_id: u64,
        query: &str,
        options: &SearchOptions,
    ) -> Vec<SearchMatch> {
        let normalized = normalize_symbols(&self.symbols, options.case_sensitive, true);
        let query_symbols = symbols_from_text(query);
        let normalized_query =
            normalize_symbols(&query_symbols, options.case_sensitive, false).value;
        find_matches(&normalized, normalized_query.trim(), options.whole_word)
            .into_iter()
            .enumerate()
            .filter_map(|(ordinal, range)| {
                let units = &normalized.units[range.first_unit..=range.last_unit];
                let source_start = units.iter().map(|unit| unit.source_start).min()?;
                let source_end = units.iter().map(|unit| unit.source_end).max()?;
                let rects = merge_rects(
                    units
                        .iter()
                        .filter_map(|unit| unit.rect.map(|rect| (unit.line, rect)))
                        .collect(),
                );
                if rects.is_empty() {
                    return None;
                }
                let page_y = rects
                    .iter()
                    .map(|rect| rect.top + rect.height / 2.0)
                    .sum::<f32>()
                    / rects.len() as f32;
                let (before_text, matched_text, after_text) =
                    context(&self.source, source_start, source_end, 32);
                Some(SearchMatch {
                    id: format!("pdf-{request_id}-{}-{ordinal}", self.page_index + 1),
                    matched_text,
                    before_text,
                    after_text,
                    location_label: format!("第 {} 页", self.page_index + 1),
                    locator: SearchLocator::Pdf(PdfSearchLocator {
                        page: self.page_index + 1,
                        page_y,
                        rects,
                    }),
                })
            })
            .collect()
    }
}

fn push_text(
    source: &mut String,
    symbols: &mut Vec<SourceSymbol>,
    text: &str,
    line: usize,
    rect: Option<SearchRect>,
) {
    for character in text.chars() {
        let start = source.len();
        source.push(character);
        symbols.push(SourceSymbol {
            text: character.to_string(),
            source_start: start,
            source_end: source.len(),
            line,
            rect,
        });
    }
}

fn symbols_from_text(value: &str) -> Vec<SourceSymbol> {
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

fn merge_rects(mut values: Vec<(usize, SearchRect)>) -> Vec<SearchRect> {
    values.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.left.total_cmp(&right.1.left))
    });
    let mut merged: Vec<(usize, SearchRect)> = Vec::new();
    for (line, rect) in values {
        if let Some((previous_line, previous)) = merged.last_mut() {
            let vertical_overlap = (previous.top + previous.height).min(rect.top + rect.height)
                - previous.top.max(rect.top);
            let same_line =
                *previous_line == line && vertical_overlap > previous.height.min(rect.height) * 0.5;
            let gap = rect.left - (previous.left + previous.width);
            if same_line && gap <= previous.height.max(rect.height) * 0.8 {
                let right = (previous.left + previous.width).max(rect.left + rect.width);
                let bottom = (previous.top + previous.height).max(rect.top + rect.height);
                previous.left = previous.left.min(rect.left);
                previous.top = previous.top.min(rect.top);
                previous.width = right - previous.left;
                previous.height = bottom - previous.top;
                continue;
            }
        }
        merged.push((line, rect));
    }
    merged.into_iter().map(|(_, rect)| rect).collect()
}

fn context(source: &str, start: usize, end: usize, radius: usize) -> (String, String, String) {
    let before_start = source[..start]
        .char_indices()
        .rev()
        .nth(radius)
        .map_or(0, |(index, _)| index);
    let after_end = source[end..]
        .char_indices()
        .nth(radius)
        .map_or(source.len(), |(index, _)| end + index);
    (
        source[before_start..start].replace('\n', " "),
        source[start..end].replace('\n', " "),
        source[end..after_end].replace('\n', " "),
    )
}

fn normalize_rect(value: Rect, page: Rect) -> SearchRect {
    SearchRect {
        left: (value.x0 - page.x0) / page.width(),
        top: (value.y0 - page.y0) / page.height(),
        width: value.width() / page.width(),
        height: value.height() / page.height(),
    }
}

fn quad_bounds(value: &Quad) -> Rect {
    let xs = [value.ul.x, value.ur.x, value.ll.x, value.lr.x];
    let ys = [value.ul.y, value.ur.y, value.ll.y, value.lr.y];
    Rect::new(
        xs.into_iter().fold(f32::INFINITY, f32::min),
        ys.into_iter().fold(f32::INFINITY, f32::min),
        xs.into_iter().fold(f32::NEG_INFINITY, f32::max),
        ys.into_iter().fold(f32::NEG_INFINITY, f32::max),
    )
}

fn mupdf_error(error: mupdf::Error) -> AppError {
    AppError::MuPdf(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_adjacent_character_rectangles_but_keeps_lines_separate() {
        let merged = merge_rects(vec![
            (
                1,
                SearchRect {
                    left: 0.10,
                    top: 0.10,
                    width: 0.02,
                    height: 0.02,
                },
            ),
            (
                1,
                SearchRect {
                    left: 0.12,
                    top: 0.10,
                    width: 0.03,
                    height: 0.02,
                },
            ),
            (
                2,
                SearchRect {
                    left: 0.10,
                    top: 0.14,
                    width: 0.04,
                    height: 0.02,
                },
            ),
        ]);
        assert_eq!(merged.len(), 2);
        assert!((merged[0].width - 0.05).abs() < f32::EPSILON);
        assert_eq!(merged[1].top, 0.14);
    }

    #[test]
    fn context_does_not_split_unicode_characters() {
        let source = "前文😀中文后文";
        let start = source.find("中文").unwrap();
        let end = start + "中文".len();
        let (_, matched, _) = context(source, start, end, 2);
        assert_eq!(matched, "中文");
    }
}
