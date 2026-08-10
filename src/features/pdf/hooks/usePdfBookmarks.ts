import { useCallback, useEffect, useRef, useState } from "react";
import type { PdfDocument } from "../../documents";
import { pdfStorageRepository } from "../repositories/pdfStorageRepository";
import type { PdfBookmark } from "../types/pdfBookmark";

type BookmarkUpdater = (items: PdfBookmark[]) => PdfBookmark[];

export function usePdfBookmarks(
  document: PdfDocument | null,
  reportError: (message: string) => void,
) {
  const [byDocument, setByDocument] = useState<Record<string, PdfBookmark[]>>({});
  const [savedByDocument, setSavedByDocument] = useState<Record<string, PdfBookmark[]>>({});
  const loadedRef = useRef(new Set<string>());
  const byDocumentRef = useRef(byDocument);
  const savedByDocumentRef = useRef(savedByDocument);
  byDocumentRef.current = byDocument;
  savedByDocumentRef.current = savedByDocument;

  useEffect(() => {
    if (!document) return;
    let cancelled = false;
    loadedRef.current.delete(document.id);
    pdfStorageRepository.loadBookmarks(document.id, document.sourcePath)
      .then((bookmarks) => {
        if (cancelled) return;
        setByDocument((current) => ({ ...current, [document.id]: bookmarks }));
        setSavedByDocument((current) => ({ ...current, [document.id]: bookmarks }));
        loadedRef.current.add(document.id);
      })
      .catch((error) => reportError(`读取书签失败：${String(error)}`));
    return () => { cancelled = true; };
  }, [document, reportError]);

  const updateBookmarks = useCallback((updater: BookmarkUpdater) => {
    if (!document) return;
    setByDocument((current) => ({
      ...current,
      [document.id]: updater(current[document.id] ?? []),
    }));
  }, [document]);

  const isDirty = useCallback((documentId: string) => {
    if (!loadedRef.current.has(documentId)) return false;
    return JSON.stringify(byDocumentRef.current[documentId] ?? [])
      !== JSON.stringify(savedByDocumentRef.current[documentId] ?? []);
  }, []);

  const persistBookmarks = useCallback(async (documentId?: string) => {
    const targetId = documentId ?? document?.id;
    if (!targetId || !loadedRef.current.has(targetId)) return;
    const current = byDocumentRef.current[targetId] ?? [];
    try {
      await pdfStorageRepository.saveBookmarks(targetId, document?.id === targetId ? document.sourcePath : undefined, current);
      setSavedByDocument((saved) => ({ ...saved, [targetId]: current }));
    } catch (error) {
      reportError(`保存书签失败：${String(error)}`);
      throw error;
    }
  }, [document?.id, reportError]);

  return {
    bookmarks: document ? byDocument[document.id] ?? [] : [],
    dirty: document ? isDirty(document.id) : false,
    isDirty,
    updateBookmarks,
    persistBookmarks,
  };
}
