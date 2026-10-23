export type SourceRange = { start: number; end: number };

export type MarkdownNode = {
  id: string;
  range: SourceRange;
} & MarkdownNodeKind;

export type MarkdownNodeKind =
  | { type: "paragraph" | "blockQuote" | "listItem" | "definitionList" | "definitionTitle" | "definition" | "tableHead" | "tableRow" | "tableCell" | "emphasis" | "strong" | "strikethrough" | "superscript" | "subscript" | "metadata"; children: MarkdownNode[] }
  | { type: "heading"; level: number; children: MarkdownNode[] }
  | { type: "codeBlock"; language?: string; children: MarkdownNode[] }
  | { type: "list"; start?: number; children: MarkdownNode[] }
  | { type: "footnoteDefinition"; label: string; children: MarkdownNode[] }
  | { type: "table"; alignments: string[]; children: MarkdownNode[] }
  | { type: "link" | "image"; url: string; title: string; children: MarkdownNode[] }
  | { type: "text" | "inlineCode" | "html"; text: string }
  | { type: "math"; text: string; display: boolean }
  | { type: "footnoteReference"; label: string }
  | { type: "taskListMarker"; checked: boolean }
  | { type: "softBreak" | "hardBreak" | "rule" };

export type MarkdownOutlineNode = {
  id: string;
  nodeId: string;
  level: number;
  title: string;
  children: MarkdownOutlineNode[];
};

export type MarkdownDiagnostic = {
  severity: string;
  message: string;
  range?: SourceRange;
};

export type StructuredMarkdownDocument = {
  documentId: string;
  sessionId: string;
  revision: number;
  path: string;
  title: string;
  source: string;
  nodes: MarkdownNode[];
  outline: MarkdownOutlineNode[];
  diagnostics: MarkdownDiagnostic[];
};

export type MarkdownEdit = { startUtf16: number; endUtf16: number; text: string };
