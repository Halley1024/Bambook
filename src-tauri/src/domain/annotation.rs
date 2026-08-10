#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Annotation {
    pub(crate) id: String,
    pub(crate) document_id: String,
    pub(crate) document_title: String,
    pub(crate) annotation_type: String,
    pub(crate) page: Option<u32>,
    pub(crate) selected_text: String,
    pub(crate) note: String,
    pub(crate) has_note: bool,
    pub(crate) color: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) rects: Option<Vec<AnnotationRect>>,
    pub(crate) area_style: Option<AreaAnnotationStyle>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AreaAnnotationStyle {
    pub(crate) background_color: String,
    pub(crate) opacity: f64,
    pub(crate) border_style: String,
    pub(crate) border_width: f64,
    pub(crate) radius: f64,
    pub(crate) top_left: bool,
    pub(crate) top_right: bool,
    pub(crate) bottom_right: bool,
    pub(crate) bottom_left: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnnotationRect {
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
}
