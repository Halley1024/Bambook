use serde::{Deserialize, Serialize};

use crate::domain::search::{
    MarkdownSearchLocator, PdfSearchLocator, SearchLocator, SearchMatch, SearchOptions, SearchRect,
    SearchRequest, SearchResponse, SearchTarget,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchRequestDto {
    request_id: u64,
    target: SearchTargetDto,
    query: String,
    #[serde(default)]
    options: SearchOptionsDto,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum SearchTargetDto {
    Pdf {
        #[serde(rename = "sessionId")]
        session_id: String,
    },
    Markdown {
        #[serde(rename = "sessionId")]
        session_id: String,
        revision: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CancelSearchRequestDto {
    request_id: u64,
    target: SearchTargetDto,
}

impl CancelSearchRequestDto {
    pub(crate) fn into_parts(self) -> (SearchTarget, u64) {
        let target = match self.target {
            SearchTargetDto::Pdf { session_id } => SearchTarget::Pdf { session_id },
            SearchTargetDto::Markdown {
                session_id,
                revision,
            } => SearchTarget::Markdown {
                session_id,
                revision,
            },
        };
        (target, self.request_id)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct SearchOptionsDto {
    case_sensitive: bool,
    whole_word: bool,
    result_limit: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchResponseDto {
    request_id: u64,
    total: usize,
    truncated: bool,
    cancelled: bool,
    matches: Vec<SearchMatchDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchMatchDto {
    id: String,
    matched_text: String,
    before_text: String,
    after_text: String,
    location_label: String,
    locator: SearchLocatorDto,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum SearchLocatorDto {
    Pdf {
        page: u32,
        #[serde(rename = "pageY")]
        page_y: f32,
        rects: Vec<SearchRectDto>,
    },
    Markdown {
        revision: u64,
        #[serde(rename = "startUtf16")]
        start_utf16: usize,
        #[serde(rename = "endUtf16")]
        end_utf16: usize,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchRectDto {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl From<SearchRequestDto> for SearchRequest {
    fn from(value: SearchRequestDto) -> Self {
        let target = match value.target {
            SearchTargetDto::Pdf { session_id } => SearchTarget::Pdf { session_id },
            SearchTargetDto::Markdown {
                session_id,
                revision,
            } => SearchTarget::Markdown {
                session_id,
                revision,
            },
        };
        Self {
            request_id: value.request_id,
            target,
            query: value.query,
            options: SearchOptions {
                case_sensitive: value.options.case_sensitive,
                whole_word: value.options.whole_word,
                result_limit: value.options.result_limit.unwrap_or(500).clamp(1, 5_000),
            },
        }
    }
}

impl From<SearchResponse> for SearchResponseDto {
    fn from(value: SearchResponse) -> Self {
        Self {
            request_id: value.request_id,
            total: value.total,
            truncated: value.truncated,
            cancelled: value.cancelled,
            matches: value.matches.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SearchMatch> for SearchMatchDto {
    fn from(value: SearchMatch) -> Self {
        let locator = match value.locator {
            SearchLocator::Pdf(PdfSearchLocator {
                page,
                page_y,
                rects,
            }) => SearchLocatorDto::Pdf {
                page,
                page_y,
                rects: rects.into_iter().map(Into::into).collect(),
            },
            SearchLocator::Markdown(MarkdownSearchLocator {
                revision,
                start_utf16,
                end_utf16,
                line,
                column,
            }) => SearchLocatorDto::Markdown {
                revision,
                start_utf16,
                end_utf16,
                line,
                column,
            },
        };
        Self {
            id: value.id,
            matched_text: value.matched_text,
            before_text: value.before_text,
            after_text: value.after_text,
            location_label: value.location_label,
            locator,
        }
    }
}

impl From<SearchRect> for SearchRectDto {
    fn from(value: SearchRect) -> Self {
        Self {
            left: value.left,
            top: value.top,
            width: value.width,
            height: value.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_tagged_pdf_request_and_applies_limit() {
        let dto: SearchRequestDto = serde_json::from_str(
            r#"{
          "requestId":7,"target":{"kind":"pdf","sessionId":"pdf-1"},"query":"全文",
          "options":{"caseSensitive":false,"wholeWord":false,"resultLimit":99999}
        }"#,
        )
        .unwrap();
        let request: SearchRequest = dto.into();
        assert_eq!(request.request_id, 7);
        assert_eq!(request.options.result_limit, 5_000);
        assert!(
            matches!(request.target, SearchTarget::Pdf { session_id } if session_id == "pdf-1")
        );
    }

    #[test]
    fn serializes_markdown_locator_with_camel_case_offsets() {
        let response = SearchResponseDto::from(SearchResponse {
            request_id: 2,
            total: 1,
            truncated: false,
            cancelled: false,
            matches: vec![SearchMatch {
                id: "m1".into(),
                matched_text: "abc".into(),
                before_text: "".into(),
                after_text: "".into(),
                location_label: "第 1 行".into(),
                locator: SearchLocator::Markdown(MarkdownSearchLocator {
                    revision: 3,
                    start_utf16: 4,
                    end_utf16: 7,
                    line: 1,
                    column: 5,
                }),
            }],
        });
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["matches"][0]["locator"]["startUtf16"], 4);
        assert_eq!(json["matches"][0]["locator"]["kind"], "markdown");
    }
}
