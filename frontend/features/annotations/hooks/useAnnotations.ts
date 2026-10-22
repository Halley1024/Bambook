import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ReaderDocument } from "../../documents";
import type { AppSettings } from "../../settings";
import { annotationRepository } from "../repositories/annotationRepository";
import type { Annotation } from "../types";

type AnnotationUpdater = (items: Annotation[]) => Annotation[];

export function useAnnotations(
  document: ReaderDocument | null,
  settings: AppSettings,
  reportError: (message: string) => void,
) {
  const [byDocument, setByDocument] = useState<Record<string, Annotation[]>>({});
  const [savedByDocument, setSavedByDocument] = useState<Record<string, Annotation[]>>({});
  const byDocumentRef = useRef(byDocument);
  const savedByDocumentRef = useRef(savedByDocument);
  const loadedKeyByDocumentRef = useRef(new Map<string, string>());
  const undoHistoryRef = useRef(new Map<string, Annotation[][]>());
  const redoHistoryRef = useRef(new Map<string, Annotation[][]>());
  const [historyRevision, setHistoryRevision] = useState(0);
  byDocumentRef.current = byDocument;
  savedByDocumentRef.current = savedByDocument;
  const annotationSettings = useMemo<Pick<AppSettings, "annotationStorageDir">>(
    () => ({ annotationStorageDir: settings.annotationStorageDir }),
    [settings.annotationStorageDir],
  );
  const documentId = document?.id ?? null;
  const sourcePath = document?.kind === "pdf" ? document.sourcePath : document?.path;
  const storageKey = documentId
    ? JSON.stringify([documentId, sourcePath ?? "", annotationSettings.annotationStorageDir ?? "default"])
    : null;
  const annotations = document ? (byDocument[document.id] ?? []) : [];

  useEffect(() => {
    if (!documentId || !storageKey) return;

    // This hook is mounted above the tab content. Keep its document cache when
    // another tab becomes active; otherwise a reload would overwrite unsaved
    // in-memory annotations with the last JSON saved on disk.
    if (loadedKeyByDocumentRef.current.get(documentId) === storageKey) return;

    let cancelled = false;
    annotationRepository
      .load(documentId, sourcePath, annotationSettings)
      .then((loaded) => {
        if (cancelled) return;
        const nextByDocument = { ...byDocumentRef.current, [documentId]: loaded };
        const nextSavedByDocument = { ...savedByDocumentRef.current, [documentId]: loaded };
        byDocumentRef.current = nextByDocument;
        savedByDocumentRef.current = nextSavedByDocument;
        setByDocument(nextByDocument);
        setSavedByDocument(nextSavedByDocument);
        undoHistoryRef.current.set(documentId, []);
        redoHistoryRef.current.set(documentId, []);
        setHistoryRevision((revision) => revision + 1);
        loadedKeyByDocumentRef.current.set(documentId, storageKey);
      })
      .catch((error) => reportError(`读取批注失败：${String(error)}`));
    return () => {
      cancelled = true;
    };
  }, [annotationSettings, documentId, reportError, sourcePath, storageKey]);

  const updateAnnotations = useCallback(
    (updater: AnnotationUpdater) => {
      if (!document) return;
      const current = byDocumentRef.current[document.id] ?? [];
      const next = updater(current);
      if (annotationsEqual(current, next)) return;
      const undo = undoHistoryRef.current.get(document.id) ?? [];
      undoHistoryRef.current.set(document.id, [...undo.slice(-99), current]);
      redoHistoryRef.current.set(document.id, []);
      const nextByDocument = { ...byDocumentRef.current, [document.id]: next };
      byDocumentRef.current = nextByDocument;
      setByDocument(nextByDocument);
      setHistoryRevision((revision) => revision + 1);
    },
    [document],
  );

  const undoAnnotations = useCallback(() => {
    if (!document) return;
    const undo = undoHistoryRef.current.get(document.id) ?? [];
    const previous = undo[undo.length - 1];
    if (!previous) return;
    const current = byDocumentRef.current[document.id] ?? [];
    undoHistoryRef.current.set(document.id, undo.slice(0, -1));
    redoHistoryRef.current.set(document.id, [...(redoHistoryRef.current.get(document.id) ?? []).slice(-99), current]);
    const next = { ...byDocumentRef.current, [document.id]: previous };
    byDocumentRef.current = next;
    setByDocument(next);
    setHistoryRevision((revision) => revision + 1);
  }, [document]);

  const redoAnnotations = useCallback(() => {
    if (!document) return;
    const redo = redoHistoryRef.current.get(document.id) ?? [];
    const restored = redo[redo.length - 1];
    if (!restored) return;
    const current = byDocumentRef.current[document.id] ?? [];
    redoHistoryRef.current.set(document.id, redo.slice(0, -1));
    undoHistoryRef.current.set(document.id, [...(undoHistoryRef.current.get(document.id) ?? []).slice(-99), current]);
    const next = { ...byDocumentRef.current, [document.id]: restored };
    byDocumentRef.current = next;
    setByDocument(next);
    setHistoryRevision((revision) => revision + 1);
  }, [document]);

  const isDirty = useCallback((documentId: string) => {
    if (!loadedKeyByDocumentRef.current.has(documentId)) return false;
    return !annotationsEqual(
      byDocumentRef.current[documentId] ?? [],
      savedByDocumentRef.current[documentId] ?? [],
    );
  }, []);

  const persistAnnotations = useCallback(async (documentId?: string) => {
    const targetId = documentId ?? document?.id;
    if (!targetId) return;
    if (!loadedKeyByDocumentRef.current.has(targetId)) return;
    const current = byDocumentRef.current[targetId] ?? [];
    try {
      const sourcePath = document?.id === targetId ? (document.kind === "pdf" ? document.sourcePath : document.path) : undefined;
      await annotationRepository.save(targetId, sourcePath, current, annotationSettings);
      setSavedByDocument((saved) => ({ ...saved, [targetId]: current }));
    } catch (error) {
      reportError(`保存批注失败：${String(error)}`);
      throw error;
    }
  }, [annotationSettings, document?.id, reportError]);

  return {
    annotations,
    dirty: document ? isDirty(document.id) : false,
    isDirty,
    updateAnnotations,
    persistAnnotations,
    undoAnnotations,
    redoAnnotations,
    canUndo: historyRevision >= 0 && Boolean(document && undoHistoryRef.current.get(document.id)?.length),
    canRedo: historyRevision >= 0 && Boolean(document && redoHistoryRef.current.get(document.id)?.length),
  };
}

function annotationsEqual(left: Annotation[], right: Annotation[]) {
  return JSON.stringify(left) === JSON.stringify(right);
}
