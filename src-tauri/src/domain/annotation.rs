#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Annotation {
    pub(crate) id: String, // 批注的唯一标识。
    pub(crate) document_id: String, // 所属文档的标识。
    pub(crate) document_title: String, // 所属文档的显示标题。
    pub(crate) annotation_type: String, // 批注类型标识，例如高亮或区域批注。
    pub(crate) page: Option<u32>, // 批注所在的 PDF 页码；不适用时为空。
    pub(crate) selected_text: String, // 被选中的原文。
    pub(crate) note: String, // 用户输入的批注内容。
    pub(crate) has_note: bool, // 是否附有批注内容。
    pub(crate) color: String, // 批注颜色。
    pub(crate) created_at: String, // 创建时间的持久化文本。
    pub(crate) updated_at: String, // 最后修改时间的持久化文本。
    pub(crate) rects: Option<Vec<AnnotationRect>>, // 批注覆盖的页面矩形区域。
    pub(crate) area_style: Option<AreaAnnotationStyle>, // 区域批注的外观设置。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AreaAnnotationStyle {
    pub(crate) background_color: String, // 区域背景颜色。
    pub(crate) opacity: f64, // 区域背景的不透明度。
    pub(crate) border_style: String, // 边框样式标识。
    pub(crate) border_width: f64, // 边框宽度。
    pub(crate) radius: f64, // 圆角半径。
    pub(crate) top_left: bool, // 是否对左上角应用圆角。
    pub(crate) top_right: bool, // 是否对右上角应用圆角。
    pub(crate) bottom_right: bool, // 是否对右下角应用圆角。
    pub(crate) bottom_left: bool, // 是否对左下角应用圆角。
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnnotationRect {
    pub(crate) left: f64, // 矩形左边界坐标。
    pub(crate) top: f64, // 矩形上边界坐标。
    pub(crate) width: f64, // 矩形宽度。
    pub(crate) height: f64, // 矩形高度。
}
