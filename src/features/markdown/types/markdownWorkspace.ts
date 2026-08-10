export type MarkdownViewMode = "realtime" | "source" | "split";

export type MarkdownFileEntry = {
  name: string;
  path: string;
  isDirectory: boolean;
  children: MarkdownFileEntry[];
};

export type MarkdownOutlineItem = {
  id: string;
  nodeId: string;
  title: string;
  level: number;
  offset: number;
};

export type MarkdownOutlineNavigationRequest = {
  nodeId: string;
  offset: number;
  requestId: number;
};

export type EditorSelection = {
  start: number;
  end: number;
};

export type EditorSelectionRequest = EditorSelection & {
  requestId: number;
};

export type MarkdownCommand =
  | "heading1"
  | "heading2"
  | "heading3"
  | "heading4"
  | "heading5"
  | "promoteHeading"
  | "demoteHeading"
  | "bold"
  | "italic"
  | "underline"
  | "strikethrough"
  | "highlight"
  | "superscript"
  | "subscript"
  | "clearFormatting"
  | "unorderedList"
  | "orderedList"
  | "taskList"
  | "inlineCode"
  | "link"
  | "image"
  | "inlineFormula"
  | "blockFormula"
  | "code"
  | "quote"
  | "table"
  | "rule";
