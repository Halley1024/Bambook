use serde::{Deserialize, Serialize};

use crate::domain::annotation::{Annotation, AnnotationRect, AreaAnnotationStyle};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnnotationDto {
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
    rects: Option<Vec<AnnotationRectDto>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    area_style: Option<AreaAnnotationStyleDto>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AreaAnnotationStyleDto {
    background_color: String,
    #[serde(default = "default_area_opacity")]
    opacity: f64,
    border_style: String,
    border_width: f64,
    radius: f64,
    rounded_corners: RoundedCornersDto,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RoundedCornersDto {
    top_left: bool,
    top_right: bool,
    bottom_right: bool,
    bottom_left: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnnotationRectDto {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

impl From<AnnotationDto> for Annotation {
    fn from(value: AnnotationDto) -> Self {
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

impl From<Annotation> for AnnotationDto {
    fn from(value: Annotation) -> Self {
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

impl From<AreaAnnotationStyleDto> for AreaAnnotationStyle {
    fn from(value: AreaAnnotationStyleDto) -> Self {
        Self { background_color: value.background_color, opacity: value.opacity, border_style: value.border_style,
            border_width: value.border_width, radius: value.radius,
            top_left: value.rounded_corners.top_left, top_right: value.rounded_corners.top_right,
            bottom_right: value.rounded_corners.bottom_right, bottom_left: value.rounded_corners.bottom_left }
    }
}

impl From<AreaAnnotationStyle> for AreaAnnotationStyleDto {
    fn from(value: AreaAnnotationStyle) -> Self {
        Self { background_color: value.background_color, opacity: value.opacity, border_style: value.border_style,
            border_width: value.border_width, radius: value.radius,
            rounded_corners: RoundedCornersDto { top_left: value.top_left, top_right: value.top_right,
                bottom_right: value.bottom_right, bottom_left: value.bottom_left } }
    }
}

fn is_false(value: &bool) -> bool {
    !value
}

fn default_area_opacity() -> f64 { 0.22 }

impl From<AnnotationRectDto> for AnnotationRect {
    fn from(value: AnnotationRectDto) -> Self {
        Self {
            left: value.left,
            top: value.top,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<AnnotationRect> for AnnotationRectDto {
    fn from(value: AnnotationRect) -> Self {
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
    use super::AnnotationDto;

    #[test]
    fn preserves_annotation_camel_case_and_type_fields() {
        let json = serde_json::json!({
            "id": "a1",
            "documentId": "doc",
            "documentTitle": "Guide",
            "type": "underline",
            "page": 3,
            "selectedText": "text",
            "note": "note",
            "color": "#ff0",
            "createdAt": "now",
            "updatedAt": "now",
            "rects": [{ "left": 0.1, "top": 0.2, "width": 0.3, "height": 0.4 }]
        });

        let dto: AnnotationDto = serde_json::from_value(json.clone()).unwrap();
        let serialized = serde_json::to_value(dto).unwrap();
        assert_eq!(serialized, json);
    }
}
