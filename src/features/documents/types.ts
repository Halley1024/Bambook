export type DocumentKind = "pdf" | "markdown";

export type BaseDocument = {
  id: string;
  title: string;
  path: string;
};

export type PdfDocument = BaseDocument & {
  kind: "pdf";
  sourcePath: string;
  sessionId: string;
  pageCount: number;
  needsPassword: boolean;
  metadata: import("../pdf/types/pdfStructure").PdfMetadata;
  outline: import("../pdf/types/pdfStructure").PdfOutlineNode[];
};

export type MarkdownDocument = BaseDocument & {
  kind: "markdown";
  sessionId: string;
  revision: number;
  markdown: string;
  nodes: import("../markdown/types/markdownAst").MarkdownNode[];
  outline: import("../markdown/types/markdownAst").MarkdownOutlineNode[];
  diagnostics: import("../markdown/types/markdownAst").MarkdownDiagnostic[];
};

export type ReaderDocument = PdfDocument | MarkdownDocument;

export type FolderEntry = {
  name: string;
  path: string;
  isDirectory: boolean;
  children: FolderEntry[];
};
