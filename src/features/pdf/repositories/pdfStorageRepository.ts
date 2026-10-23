import { invokeCommand } from "../../../platform/tauriClient";
import type { PdfBookmark } from "../types/pdfBookmark";

export type PdfReadingState = {
  page: number;
  pageY?: number;
  zoomMode: "custom" | "fit-width" | "fit-page";
  scale: number;
  updatedAt: string;
};

export const pdfStorageRepository = {
  persistDocument(sessionId: string) {
    return invokeCommand<void>("persist_pdf_document", { sessionId });
  },
  loadBookmarks(documentId: string, sourcePath?: string) {
    return invokeCommand<PdfBookmark[]>("load_pdf_bookmarks", { documentId, sourcePath });
  },

  saveBookmarks(documentId: string, sourcePath: string | undefined, bookmarks: PdfBookmark[]) {
    return invokeCommand<void>("save_pdf_bookmarks", { documentId, sourcePath, bookmarks });
  },

  loadReadingState(documentId: string, sourcePath?: string) {
    return invokeCommand<PdfReadingState | null>("load_pdf_reading_state", { documentId, sourcePath });
  },

  saveReadingState(documentId: string, sourcePath: string | undefined, readingState: PdfReadingState) {
    return invokeCommand<void>("save_pdf_reading_state", { documentId, sourcePath, readingState });
  },
};
