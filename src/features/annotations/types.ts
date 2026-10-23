export type MarkupType = "highlight" | "underline" | "squiggly" | "strikeout";
export type AnnotationType = "note" | "area" | MarkupType;

export type AreaBorderStyle = "dotted" | "dashed" | "solid" | "none";
export type AreaCorner = "topLeft" | "topRight" | "bottomRight" | "bottomLeft";
export type AreaAnnotationStyle = {
  backgroundColor: string;
  opacity: number;
  borderStyle: AreaBorderStyle;
  borderWidth: number;
  radius: number;
  roundedCorners: Record<AreaCorner, boolean>;
};

export type AnnotationRect = {
  left: number;
  top: number;
  width: number;
  height: number;
};

export type Annotation = {
  id: string;
  documentId: string;
  documentTitle: string;
  type: AnnotationType;
  page?: number;
  selectedText: string;
  note: string;
  hasNote?: boolean;
  color: string;
  createdAt: string;
  updatedAt: string;
  rects?: AnnotationRect[];
  areaStyle?: AreaAnnotationStyle;
};
