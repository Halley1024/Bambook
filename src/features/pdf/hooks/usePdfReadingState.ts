import { useCallback, useEffect, useRef, useState } from "react";
import type { PdfDocument } from "../../documents";
import { pdfStorageRepository, type PdfReadingState } from "../repositories/pdfStorageRepository";
import type { PdfZoomMode } from "../state/PdfWorkspaceContext";

export type PdfReadingSnapshot = {
  page: number;
  pageY: number;
  zoomMode: PdfZoomMode;
  scale: number;
};

const DEFAULT_READING_STATE: PdfReadingSnapshot = {
  page: 1,
  pageY: 0,
  zoomMode: "fit-width",
  scale: 1.5,
};

export function usePdfReadingState(
  document: PdfDocument | null,
  snapshot: PdfReadingSnapshot,
  restore: (snapshot: PdfReadingSnapshot) => void,
  reportError: (message: string) => void,
) {
  const [readyRevision, setReadyRevision] = useState(0);
  const readyDocumentRef = useRef<string | null>(null);
  const snapshotsRef = useRef(new Map<string, PdfReadingSnapshot>());
  const timersRef = useRef(new Map<string, number>());

  const flush = useCallback(async (documentId: string) => {
    const timer = timersRef.current.get(documentId);
    if (timer !== undefined) {
      window.clearTimeout(timer);
      timersRef.current.delete(documentId);
    }
    const current = snapshotsRef.current.get(documentId);
    if (!current) return;
    const readingState: PdfReadingState = {
      ...current,
      updatedAt: new Date().toISOString(),
    };
    try {
      await pdfStorageRepository.saveReadingState(documentId, document?.id === documentId ? document.sourcePath : undefined, readingState);
    } catch (error) {
      reportError(`保存 PDF 阅读位置失败：${String(error)}`);
      throw error;
    }
  }, [document, reportError]);

  useEffect(() => {
    if (!document) {
      readyDocumentRef.current = null;
      return;
    }
    const documentId = document.id;
    let cancelled = false;
    let readyTimer = 0;
    readyDocumentRef.current = null;
    pdfStorageRepository.loadReadingState(documentId, document.sourcePath)
      .then((stored) => {
        if (cancelled) return;
        const restored = normalizeReadingState(stored, document.pageCount);
        snapshotsRef.current.set(documentId, restored);
        restore(restored);
        readyTimer = window.setTimeout(() => {
          if (cancelled) return;
          readyDocumentRef.current = documentId;
          setReadyRevision((revision) => revision + 1);
        }, 0);
      })
      .catch((error) => {
        if (cancelled) return;
        reportError(`读取 PDF 阅读位置失败：${String(error)}`);
        const fallback = normalizeReadingState(null, document.pageCount);
        snapshotsRef.current.set(documentId, fallback);
        restore(fallback);
        readyTimer = window.setTimeout(() => {
          if (cancelled) return;
          readyDocumentRef.current = documentId;
          setReadyRevision((revision) => revision + 1);
        }, 0);
      });
    return () => {
      cancelled = true;
      window.clearTimeout(readyTimer);
      if (readyDocumentRef.current === documentId) {
        readyDocumentRef.current = null;
        void flush(documentId).catch(() => undefined);
      }
    };
  }, [document, flush, reportError, restore]);

  useEffect(() => {
    const documentId = document?.id;
    if (!documentId || readyDocumentRef.current !== documentId) return;
    snapshotsRef.current.set(documentId, snapshot);
    const previousTimer = timersRef.current.get(documentId);
    if (previousTimer !== undefined) window.clearTimeout(previousTimer);
    const timer = window.setTimeout(() => {
      timersRef.current.delete(documentId);
      void flush(documentId).catch(() => undefined);
    }, 700);
    timersRef.current.set(documentId, timer);
    return () => window.clearTimeout(timer);
  }, [document?.id, flush, readyRevision, snapshot.page, snapshot.pageY, snapshot.scale, snapshot.zoomMode]);

  return { flushReadingState: flush };
}

function normalizeReadingState(
  stored: PdfReadingState | null,
  pageCount: number,
): PdfReadingSnapshot {
  if (!stored) return DEFAULT_READING_STATE;
  const zoomMode = stored.zoomMode === "custom" || stored.zoomMode === "fit-page"
    || stored.zoomMode === "fit-width"
    ? stored.zoomMode
    : DEFAULT_READING_STATE.zoomMode;
  return {
    page: Math.max(1, Math.min(Math.max(1, pageCount), Math.round(stored.page))),
    pageY: clampNumber(stored.pageY ?? 0, 0, 1),
    zoomMode,
    scale: clampNumber(stored.scale, 0.25, 5),
  };
}

function clampNumber(value: number, minimum: number, maximum: number) {
  return Number.isFinite(value) ? Math.max(minimum, Math.min(maximum, value)) : minimum;
}
