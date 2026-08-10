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
  const loadedKeysRef = useRef(new Set<string>());
  const undoHistoryRef = useRef(new Map<string, Annotation[][]>());
  const redoHistoryRef = useRef(new Map<string, Annotation[][]>());
  const [historyRevision, setHistoryRevision] = useState(0);
  byDocumentRef.current = byDocument;
  savedByDocumentRef.current = savedByDocument;
  const annotationSettings = useMemo<Pick<AppSettings, "annotationStorageDir">>(
    () => ({ annotationStorageDir: settings.annotationStorageDir }),
    [settings.annotationStorageDir],
  );
  const storageKey = document
    ? `${document.id}::${document.path}::${annotationSettings.annotationStorageDir ?? "default"}`
    : null;
  const annotations = document ? (byDocument[document.id] ?? []) : [];

  useEffect(() => {
    if (!document || !storageKey) return;
    loadedKeysRef.current.delete(storageKey);
    let cancelled = false;
    annotationRepository
      .load(document.id, document.kind === "pdf" ? document.sourcePath : document.path, annotationSettings)
      .then((loaded) => {
        if (cancelled) return;
        setByDocument((current) => ({ ...current, [document.id]: loaded }));
        setSavedByDocument((current) => ({ ...current, [document.id]: loaded }));
        undoHistoryRef.current.set(document.id, []);
        redoHistoryRef.current.set(document.id, []);
        setHistoryRevision((revision) => revision + 1);
        loadedKeysRef.current.add(storageKey);
      })
      .catch((error) => reportError(`读取批注失败：${String(error)}`));
    return () => {
      cancelled = true;
    };
  }, [annotationSettings, document, reportError, storageKey]);

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
    if (![...loadedKeysRef.current].some((key) => key.startsWith(`${documentId}::`))) return false;
    return !annotationsEqual(
      byDocumentRef.current[documentId] ?? [],
      savedByDocumentRef.current[documentId] ?? [],
    );
  }, []);

  const persistAnnotations = useCallback(async (documentId?: string) => {
    const targetId = documentId ?? document?.id;
    if (!targetId) return;
    if (![...loadedKeysRef.current].some((key) => key.startsWith(`${targetId}::`))) return;
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
