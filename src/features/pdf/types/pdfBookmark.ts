export type PdfBookmark = {
  id: string;
  documentId: string;
  page: number;
  x: number;
  y: number;
  title: string;
  color: string;
  createdAt: string;
};

export type PdfBookmarkDraft = Pick<PdfBookmark, "page" | "x" | "y"> & { title?: string; color?: string };
