export type PdfRect = { left: number; top: number; right: number; bottom: number };
export type PdfMetadata = { title: string; author: string; subject: string; keywords: string; creator: string; producer: string };
export type PdfOutlineNode = { title: string; page?: number; x?: number; y?: number; uri?: string; children: PdfOutlineNode[] };
export type MupdfDocument = { documentId: string; sessionId: string; path: string; sourcePath: string; title: string; pageCount: number; needsPassword: boolean; metadata: PdfMetadata; outline: PdfOutlineNode[] };
export type PdfTextSpan = { text: string; bounds: PdfRect; fontSize: number };
export type PdfTextLine = { bounds: PdfRect; spans: PdfTextSpan[] };
export type PdfTextBlock = { bounds: PdfRect; lines: PdfTextLine[] };
export type PdfPageLink = {
  bounds: PdfRect;
  uri: string;
  /** One-based target page for links resolved inside the current document. */
  targetPage?: number;
  targetX?: number;
  targetY?: number;
};
export type PdfPageStructure = { pageIndex: number; width: number; height: number; blocks: PdfTextBlock[]; links: PdfPageLink[] };
