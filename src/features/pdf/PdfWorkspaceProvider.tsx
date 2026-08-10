import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { annotationRepository, useAnnotations, type Annotation, type AnnotationRect, type MarkupType } from "../annotations";
import { useDocumentWorkspace, type PdfDocument } from "../documents";
import { useNotification } from "../notifications";
import { useSettings } from "../settings";
import { PdfWorkspaceContext, type PdfInteractionMode, type PdfTextSelection, type PdfZoomMode } from "./state/PdfWorkspaceContext";
import { mupdfRepository } from "./repositories/mupdfRepository";
import { usePdfBookmarks } from "./hooks/usePdfBookmarks";
import { usePdfReadingState, type PdfReadingSnapshot } from "./hooks/usePdfReadingState";
import type { PdfNavigationTarget } from "./types/pdfNavigation";
import type { PdfBookmark, PdfBookmarkDraft } from "./types/pdfBookmark";
import { selectDirectoryPath } from "../../platform/fileDialog";
import { documentLibraryRepository, notifyDocumentLibraryChanged } from "../documents/repositories/documentLibraryRepository";
import { applyRelinkedDocument, ensureDocumentSource } from "../documents/repositories/documentRelink";
import { pdfStorageRepository } from "./repositories/pdfStorageRepository";
import { openExternalUrl } from "../../platform/externalLink";

type HistoryLocation = Pick<PdfNavigationTarget, "page" | "pageX" | "pageY">;

export function PdfWorkspaceProvider({ children }: { children: ReactNode }) {
  const documentWorkspace = useDocumentWorkspace();
  const documents = documentWorkspace.documents.filter((document): document is PdfDocument => document.kind === "pdf");
  const activeDocument = documentWorkspace.activeDocument?.kind === "pdf" ? documentWorkspace.activeDocument : null;
  const activeDocumentId = activeDocument?.id ?? null;
  const { settings, persistSettings } = useSettings();
  const notification = useNotification();
  const [scale, setScale] = useState(1.5);
  const [zoomMode, setZoomMode] = useState<PdfZoomMode>("fit-width");
  const [currentPage, setCurrentPage] = useState(1);
  const [currentPageY, setCurrentPageY] = useState(0);
  const [currentPageX, setCurrentPageX] = useState(0.5);
  const [navigationTarget, setNavigationTarget] = useState<PdfNavigationTarget | null>(null);
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);
  const [annotationPanelCollapsed, setAnnotationPanelCollapsed] = useState(false);
  const [selectedAnnotationId, setSelectedAnnotationId] = useState<string | null>(null);
  const [activeMarkupType, setActiveMarkupType] = useState<MarkupType>("underline");
  const [interactionMode, setInteractionMode] = useState<PdfInteractionMode>("pointer");
  const [activeAnnotationColor, setActiveAnnotationColor] = useState("#26765A");
  const [movingBookmarkId, setMovingBookmarkId] = useState<string | null>(null);
  const [transformingAreaId, setTransformingAreaId] = useState<string | null>(null);
  const [selectionSnapshot, setSelectionSnapshot] = useState<PdfTextSelection | null>(null);
  const [historyRevision, setHistoryRevision] = useState(0);
  const historyBackRef = useRef<HistoryLocation[]>([]);
  const historyForwardRef = useRef<HistoryLocation[]>([]);
  const outlineNavigationRequestRef = useRef(0);
  const { annotations, dirty: annotationsDirty, isDirty: annotationsAreDirty, updateAnnotations, persistAnnotations,
    undoAnnotations, redoAnnotations, canUndo: canUndoAnnotations, canRedo: canRedoAnnotations } = useAnnotations(
    activeDocument,
    settings,
    notification.error,
  );
  const {
    bookmarks,
    dirty: bookmarksDirty,
    isDirty: bookmarksAreDirty,
    updateBookmarks,
    persistBookmarks,
  } = usePdfBookmarks(activeDocument, notification.error);
  const dirty = annotationsDirty || bookmarksDirty;
  useEffect(() => {
    const handleAnnotationHistoryShortcut = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
      const target = event.target instanceof HTMLElement ? event.target : null;
      if (target?.matches("input, textarea, [contenteditable='true']")) return;
      if (event.key.toLocaleLowerCase() === "z") {
        event.preventDefault();
        if (event.shiftKey) redoAnnotations(); else undoAnnotations();
      } else if (event.key.toLocaleLowerCase() === "y") {
        event.preventDefault(); redoAnnotations();
      }
    };
    window.addEventListener("keydown", handleAnnotationHistoryShortcut);
    return () => window.removeEventListener("keydown", handleAnnotationHistoryShortcut);
  }, [redoAnnotations, undoAnnotations]);
  const restoreReadingState = useCallback((readingState: PdfReadingSnapshot) => {
    setScale(readingState.scale);
    setZoomMode(readingState.zoomMode);
    setCurrentPage(readingState.page);
    setCurrentPageY(readingState.pageY);
    setNavigationTarget({
      page: readingState.page,
      pageY: readingState.pageY,
      requestId: Date.now(),
    });
  }, []);
  const { flushReadingState } = usePdfReadingState(
    activeDocument,
    { page: currentPage, pageY: currentPageY, zoomMode, scale },
    restoreReadingState,
    notification.error,
  );
  const handleViewportChange = useCallback((page: number, pageY = 0, pageX = 0.5) => {
    setCurrentPage(page);
    setCurrentPageY(pageY);
    setCurrentPageX(pageX);
  }, []);

  useEffect(() => {
    if (!activeDocument) return;
    documentWorkspace.setDocumentDirty(activeDocument.id, dirty);
  }, [activeDocument, dirty, documentWorkspace.setDocumentDirty]);

  useEffect(() => {
    if (!documentWorkspace.saveActiveRequest.revision || !activeDocument
      || documentWorkspace.saveActiveRequest.documentId !== activeDocument.id) return;
    void saveActiveDocument();
  }, [documentWorkspace.saveActiveRequest.revision]);

  useEffect(() => {
    if (!activeDocumentId) return;
    const handleSaveShortcut = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.altKey || event.shiftKey) return;
      if (event.key.toLocaleLowerCase() !== "s") return;
      event.preventDefault();
      if (event.repeat) return;
      documentWorkspace.requestSaveActive?.(activeDocumentId);
    };
    window.addEventListener("keydown", handleSaveShortcut);
    return () => window.removeEventListener("keydown", handleSaveShortcut);
  }, [activeDocumentId, documentWorkspace.requestSaveActive]);

  useEffect(() => {
    outlineNavigationRequestRef.current += 1;
    setCurrentPage(1);
    setCurrentPageY(0);
    setNavigationTarget(null);
    setSelectedAnnotationId(null);
    setSelectionSnapshot(null);
    setInteractionMode("pointer");
    setMovingBookmarkId(null);
    setTransformingAreaId(null);
    historyBackRef.current = [];
    historyForwardRef.current = [];
    setHistoryRevision((revision) => revision + 1);
  }, [activeDocumentId]);

  useEffect(() => documentWorkspace.registerLifecycle("pdf", {
    requiresCloseDecision: (document) => document.kind === "pdf"
      && (annotationsAreDirty(document.id) || bookmarksAreDirty(document.id)),
    prepareSave: async (document) => document.kind === "pdf"
      ? await ensureSourceBeforeSave(document) ? { status: "ready", commit: async () => {
        flushNoteEditors();
        await pdfStorageRepository.persistDocument(document.sessionId);
        notifyDocumentLibraryChanged();
        await Promise.all([persistAnnotations(document.id), persistBookmarks(document.id)]);
      } } : { status: "cancelled" }
      : { status: "ready" },
    close: async (document) => {
      if (document.kind !== "pdf") return;
      await flushReadingState(document.id).catch(() => undefined);
      await documentLibraryRepository.reconcileStorage(document.id);
      notifyDocumentLibraryChanged();
      await persistSettings();
      await mupdfRepository.close(document.sessionId);
    },
  }), [annotationsAreDirty, bookmarksAreDirty, documentWorkspace.registerLifecycle, flushReadingState, persistAnnotations, persistBookmarks, persistSettings]);

  async function saveActiveDocument() {
    if (!activeDocument) {
      notification.warning("请先打开 PDF 文档。");
      return;
    }
    try {
      if (!await ensureSourceBeforeSave(activeDocument)) return;
      flushNoteEditors();
      await pdfStorageRepository.persistDocument(activeDocument.sessionId);
      notifyDocumentLibraryChanged();
      await Promise.all([
        persistAnnotations(activeDocument.id),
        persistBookmarks(activeDocument.id),
      ]);
      documentWorkspace.setDocumentDirty(activeDocument.id, false);
      notification.success(`已保存批注：${activeDocument.title}`);
    } catch (error) {
      notification.error(`保存 PDF 失败：${String(error)}`);
    }
  }

  async function exportNotes() {
    if (!activeDocument) {
      notification.warning("请先导入 PDF 文档。");
      return;
    }
    try {
      flushNoteEditors();
      await pdfStorageRepository.persistDocument(activeDocument.sessionId);
      notifyDocumentLibraryChanged();
      const path = await annotationRepository.exportMarkdown(activeDocument, annotations, settings);
      notification.success(`已导出：${path}`);
    } catch (error) {
      notification.error(`导出失败：${String(error)}`);
    }
  }

  async function saveActiveDocumentAs() {
    if (!activeDocument) return;
    if (!await ensureSourceBeforeSave(activeDocument)) return;
    const destination = await selectDirectoryPath();
    if (!destination) return;
    try {
      flushNoteEditors();
      await pdfStorageRepository.persistDocument(activeDocument.sessionId);
      notifyDocumentLibraryChanged();
      await Promise.all([persistAnnotations(activeDocument.id), persistBookmarks(activeDocument.id), flushReadingState(activeDocument.id)]);
      const packagePath = await documentLibraryRepository.exportPackage(activeDocument.id, destination);
      notifyDocumentLibraryChanged();
      documentWorkspace.setDocumentDirty(activeDocument.id, false);
      notification.success(`文档包已保存到：${packagePath}`);
    } catch (error) {
      notification.error(`另存为 Bambook 文档包失败：${String(error)}`);
    }
  }

  async function ensureSourceBeforeSave(document: PdfDocument) {
    const result = await ensureDocumentSource(document);
    if (result.status === "cancelled") return false;
    if (result.status === "relinked") {
      documentWorkspace.replaceDocument(applyRelinkedDocument(document, result.document));
    }
    return true;
  }

  function applyMarkup(type: MarkupType, explicitSelection?: PdfTextSelection, withNote = false) {
    const document = activeDocument;
    if (!document) {
      notification.warning("请先打开 PDF 文档。");
      return;
    }
    const selectedAnnotation = annotations.find((annotation) => annotation.id === selectedAnnotationId);
    if (selectedAnnotation) {
      convertAnnotation(selectedAnnotation.id, type);
      return;
    }
    const selection = explicitSelection ?? selectionSnapshot;
    if (!selection?.text.trim() || !selection.rects.length) {
      notification.warning("请先选中文本。");
      return;
    }
    const now = new Date().toISOString();
    const annotationId = crypto.randomUUID();
    updateAnnotations((items) => [
      {
        id: annotationId,
        documentId: document.id,
        documentTitle: document.title,
        type,
        page: selection.page,
        selectedText: selection.text,
        note: "",
        hasNote: withNote,
        color: activeAnnotationColor,
        createdAt: now,
        updatedAt: now,
        rects: selection.rects,
      },
      ...items,
    ]);
    setSelectionSnapshot(null);
    setSelectedAnnotationId(withNote ? annotationId : null);
    setActiveMarkupType(type);
    if (withNote) setAnnotationPanelCollapsed(false);
    window.getSelection()?.removeAllRanges();
    notification.success(`已在第 ${selection.page} 页添加${markupLabel(type)}。`);
  }

  function addAreaAnnotation(page: number, rect: AnnotationRect) {
    const document = activeDocument;
    if (!document) return;
    const now = new Date().toISOString();
    const id = crypto.randomUUID();
    updateAnnotations((items) => [{
      id,
      documentId: document.id,
      documentTitle: document.title,
      type: "area",
      page,
      selectedText: "选择区域",
      note: "",
      color: activeAnnotationColor,
      createdAt: now,
      updatedAt: now,
      rects: [rect],
      areaStyle: {
        backgroundColor: activeAnnotationColor,
        opacity: 0.22,
        borderStyle: "solid",
        borderWidth: 2,
        radius: 6,
        roundedCorners: { topLeft: true, topRight: true, bottomRight: true, bottomLeft: true },
      },
    }, ...items]);
    setSelectedAnnotationId(id);
    setInteractionMode("pointer");
    notification.success(`已在第 ${page} 页添加选择区域。`);
  }

  function selectAnnotation(id: string | null) {
    setSelectedAnnotationId(id);
    if (!id) return;
    const annotation = annotations.find((item) => item.id === id);
    if (!annotation) return;
    if (isMarkupType(annotation.type)) setActiveMarkupType(annotation.type);
    setActiveAnnotationColor(annotation.color);
    setAnnotationPanelCollapsed(false);
  }

  function navigate(target: HistoryLocation, recordHistory = true) {
    if (!activeDocument) return;
    if (recordHistory) {
      const current = { page: currentPage, pageX: currentPageX, pageY: currentPageY };
      const previous = historyBackRef.current[historyBackRef.current.length - 1];
      if (!sameHistoryLocation(previous, current) && !sameHistoryLocation(current, target)) {
        historyBackRef.current.push(current);
        if (historyBackRef.current.length > 100) historyBackRef.current.shift();
      }
      historyForwardRef.current = [];
    }
    setCurrentPage(target.page);
    setCurrentPageX(target.pageX ?? 0.5);
    setCurrentPageY(target.pageY ?? 0);
    setNavigationTarget({ ...target, requestId: Date.now() });
    setHistoryRevision((revision) => revision + 1);
  }

  function navigateHistoryBack() {
    const target = historyBackRef.current.pop();
    if (!target) return;
    historyForwardRef.current.push({ page: currentPage, pageX: currentPageX, pageY: currentPageY });
    navigate(target, false);
  }

  function navigateHistoryForward() {
    const target = historyForwardRef.current.pop();
    if (!target) return;
    historyBackRef.current.push({ page: currentPage, pageX: currentPageX, pageY: currentPageY });
    navigate(target, false);
  }

  function navigateToAnnotation(id: string) {
    selectAnnotation(id);
    const annotation = annotations.find((item) => item.id === id);
    if (!annotation?.page) return;
    navigate({
      page: annotation.page,
      pageY: annotationVerticalCenter(annotation),
    });
  }

  async function navigateToOutline(page: number, title: string) {
    const document = activeDocument;
    if (!document) return;
    const requestId = ++outlineNavigationRequestRef.current;
    let pageY = 0.5;
    try {
      const structure = await mupdfRepository.pageStructure(document.sessionId, page - 1);
      const targetLine = findOutlineTitleLine(structure.blocks.flatMap((block) => block.lines), title);
      if (targetLine && structure.height > 0) {
        pageY = (targetLine.bounds.top + targetLine.bounds.bottom) / 2 / structure.height;
      }
    } catch {
      // A page-center fallback still preserves outline navigation when text extraction is unavailable.
    }
    if (requestId !== outlineNavigationRequestRef.current) return;
    navigate({ page, pageY });
  }

  function convertAnnotation(id: string, type: MarkupType) {
    updateAnnotations((items) => items.map((annotation) => annotation.id === id
      ? { ...annotation, type, updatedAt: new Date().toISOString() } : annotation));
    setSelectedAnnotationId(id);
    setActiveMarkupType(type);
  }

  function setAnnotationColor(color: string) {
    setActiveAnnotationColor(color);
    if (!selectedAnnotationId) return;
    const annotationId = selectedAnnotationId;
    updateAnnotations((items) => items.map((annotation) => annotation.id === selectedAnnotationId
      ? { ...annotation, color, updatedAt: new Date().toISOString() } : annotation));
    window.requestAnimationFrame(() => annotationColorTargets(annotationId)
      .forEach((element) => element.style.removeProperty("--annotation-color")));
  }

  function previewAnnotationColor(color: string) {
    if (!selectedAnnotationId) return;
    annotationColorTargets(selectedAnnotationId)
      .forEach((element) => element.style.setProperty("--annotation-color", color));
  }

  function updateAnnotation(annotation: Annotation) {
    updateAnnotations((items) => items.map((item) => (item.id === annotation.id ? annotation : item)));
  }

  function beginAreaTransform(id: string | null) {
    setTransformingAreaId(id);
    if (id) {
      setMovingBookmarkId(null);
      setInteractionMode("pointer");
    }
  }

  function beginBookmarkMove(id: string | null) {
    setMovingBookmarkId(id);
    if (id) {
      setTransformingAreaId(null);
      setInteractionMode("pointer");
    }
  }

  function mergeAnnotations(ids: string[]) {
    const selected = annotations.filter((annotation) => ids.includes(annotation.id))
      .sort((left, right) => (left.page ?? 0) - (right.page ?? 0)
        || (left.rects?.[0]?.top ?? 0) - (right.rects?.[0]?.top ?? 0));
    if (selected.length < 2) return;
    if (selected.some((annotation) => annotation.type === "area")) {
      notification.warning("选择区域不能与文字批注合并。");
      return;
    }
    const page = selected[0].page;
    if (selected.some((annotation) => annotation.page !== page)) {
      notification.warning("只能合并同一页中的批注笔记。");
      return;
    }
    const primary = selected[0];
    const selectedIds = new Set(selected.map((annotation) => annotation.id));
    const merged: Annotation = {
      ...primary,
      selectedText: selected.map((annotation) => annotation.selectedText.trim()).filter(Boolean).join("\n"),
      note: selected.map((annotation) => annotation.note.trim()).filter(Boolean).join("\n\n"),
      hasNote: selected.some((annotation) => annotation.hasNote || annotation.note.trim()),
      rects: selected.flatMap((annotation) => annotation.rects ?? []),
      updatedAt: new Date().toISOString(),
    };
    updateAnnotations((items) => items.filter((annotation) => !selectedIds.has(annotation.id) || annotation.id === primary.id)
      .map((annotation) => annotation.id === primary.id ? merged : annotation));
    setSelectedAnnotationId(primary.id);
    notification.success(`已合并 ${selected.length} 条批注笔记`);
  }

  function addBookmark(draft: PdfBookmarkDraft) {
    if (!activeDocument) return;
    const documentId = activeDocument.id;
    updateBookmarks((documentBookmarks) => {
      const ordinal = documentBookmarks.length + 1;
      const bookmark: PdfBookmark = {
        page: draft.page,
        x: draft.x,
        y: draft.y,
        id: crypto.randomUUID(),
        documentId,
        title: draft.title?.trim() || `第 ${draft.page} 页书签 ${ordinal}`,
        color: draft.color ?? "#26765A",
        createdAt: new Date().toISOString(),
      };
      return [...documentBookmarks, bookmark];
    });
    notification.success(`已在第 ${draft.page} 页添加书签。`);
  }

  function updateBookmark(id: string, changes: Partial<Pick<PdfBookmark, "title" | "color" | "page" | "x" | "y">>) {
    if (!activeDocumentId) return;
    const normalized = { ...changes, ...(changes.title === undefined ? {} : { title: changes.title.trim() }) };
    if (normalized.title === "") return;
    updateBookmarks((items) =>
      items.map((bookmark) => bookmark.id === id
        ? { ...bookmark, ...normalized }
        : bookmark));
  }

  function deleteBookmark(id: string) {
    if (!activeDocumentId) return;
    updateBookmarks((items) => items.filter((bookmark) => bookmark.id !== id));
  }

  function clearBookmarks(page?: number) {
    if (!activeDocumentId) return;
    updateBookmarks((items) => page === undefined ? [] : items.filter((bookmark) => bookmark.page !== page));
  }

  function navigateToBookmark(id: string) {
    const bookmark = bookmarks.find((item) => item.id === id);
    if (bookmark) navigate({ page: bookmark.page, pageX: bookmark.x, pageY: bookmark.y });
  }

  async function openExternalLink(uri: string) {
    try {
      await openExternalUrl(uri);
    } catch (error) {
      notification.error(`打开外部链接失败：${String(error)}`);
    }
  }

  function selectScale(nextScale: number) {
    setZoomMode("custom");
    setScale(nextScale);
  }

  const value = {
      documents,
      activeDocument,
      activeDocumentId,
      annotations,
      bookmarks,
      movingBookmarkId,
      transformingAreaId,
      scale,
      zoomMode,
      currentPage,
      navigationTarget,
      sidebarCollapsed,
      annotationPanelCollapsed,
      selectedAnnotationId,
      activeMarkupType,
      interactionMode,
      activeAnnotationColor,
      canNavigateHistoryBack: historyRevision >= 0 && historyBackRef.current.length > 0,
      canNavigateHistoryForward: historyRevision >= 0 && historyForwardRef.current.length > 0,
      canUndoAnnotations,
      canRedoAnnotations,
      openFile: documentWorkspace.openFile,
      saveActiveDocument,
      saveActiveDocumentAs,
      exportNotes,
      activateDocument: documentWorkspace.activateDocument,
      closeDocument: (id: string) => void documentWorkspace.closeDocument(id),
      closeOtherDocuments: (id: string) => void documentWorkspace.closeOtherDocuments(id),
      reorderDocuments: documentWorkspace.reorderDocuments,
      copyDocumentPath: documentWorkspace.copyDocumentPath,
      setScale: selectScale,
      fitToWidth: () => { setZoomMode("fit-width"); setScale(1.5); },
      fitToPage: () => setZoomMode("fit-page"),
      resolveFitScale: setScale,
      setCurrentPage: handleViewportChange,
      navigateToPage: (page: number) => navigate({ page }),
      navigateToOutline,
      navigateToSearchResult: (page: number, pageY: number) => navigate({ page, pageY }),
      navigateHistoryBack,
      navigateHistoryForward,
      toggleSidebar: () => setSidebarCollapsed((collapsed) => !collapsed),
      toggleAnnotationPanel: () => setAnnotationPanelCollapsed((collapsed) => !collapsed),
      captureSelection: (selection: PdfTextSelection) => {
        setSelectionSnapshot(selection);
        setSelectedAnnotationId(null);
      },
      clearSelection: () => {
        setSelectionSnapshot(null);
        setSelectedAnnotationId(null);
      },
      applyMarkup,
      selectPointerTool: () => { setInteractionMode("pointer"); setTransformingAreaId(null); setMovingBookmarkId(null); },
      selectPanTool: () => { setInteractionMode("pan"); setMovingBookmarkId(null); setTransformingAreaId(null); setSelectionSnapshot(null); window.getSelection()?.removeAllRanges(); },
      selectMarkupTool: (type: MarkupType) => {
        setActiveMarkupType(type);
        setInteractionMode("markup");
        setMovingBookmarkId(null);
        setTransformingAreaId(null);
        setSelectedAnnotationId(null);
      },
      selectAreaTool: () => {
        setInteractionMode("area");
        setMovingBookmarkId(null);
        setTransformingAreaId(null);
        setSelectedAnnotationId(null);
        setSelectionSnapshot(null);
        window.getSelection()?.removeAllRanges();
      },
      selectEraserTool: () => { setInteractionMode("eraser"); setMovingBookmarkId(null); setTransformingAreaId(null); setSelectionSnapshot(null); window.getSelection()?.removeAllRanges(); },
      addNote: (selection?: PdfTextSelection) => applyMarkup(activeMarkupType, selection, true),
      selectAnnotation,
      navigateToAnnotation,
      previewAnnotationColor,
      setAnnotationColor,
      convertAnnotation,
      updateAnnotation,
      addAreaAnnotation,
      beginAreaTransform,
      deleteAnnotation: (id: string) => {
        updateAnnotations((items) => items.filter((item) => item.id !== id));
        if (selectedAnnotationId === id) setSelectedAnnotationId(null);
        if (transformingAreaId === id) setTransformingAreaId(null);
      },
      undoAnnotations,
      redoAnnotations,
      mergeAnnotations,
      addBookmark,
      updateBookmark,
      deleteBookmark,
      clearBookmarks,
      beginBookmarkMove,
      navigateToBookmark,
      navigateToLink: (page: number, pageX?: number, pageY?: number) => navigate({ page, pageX, pageY }),
      openExternalLink,
  };

  return <PdfWorkspaceContext.Provider value={value}>{children}</PdfWorkspaceContext.Provider>;
}

function isMarkupType(type: Annotation["type"]): type is MarkupType {
  return type === "highlight" || type === "underline" || type === "squiggly" || type === "strikeout";
}

function flushNoteEditors() {
  window.dispatchEvent(new Event("bambook:flush-note-editors"));
}

function markupLabel(type: MarkupType) {
  if (type === "highlight") return "高亮";
  return type === "underline" ? "下划线" : type === "squiggly" ? "波浪线" : "删除线";
}

function sameHistoryLocation(left: HistoryLocation | undefined, right: HistoryLocation) {
  return left?.page === right.page && left.pageX === right.pageX && left.pageY === right.pageY;
}

function annotationVerticalCenter(annotation: Annotation) {
  if (!annotation.rects?.length) return undefined;
  const top = Math.min(...annotation.rects.map((rect) => rect.top));
  const bottom = Math.max(...annotation.rects.map((rect) => rect.top + rect.height));
  return (top + bottom) / 2;
}

function findOutlineTitleLine<T extends { spans: { text: string }[]; bounds: { top: number; bottom: number } }>(lines: T[], title: string) {
  const normalizedTitle = normalizeOutlineText(title);
  if (!normalizedTitle) return undefined;
  const candidates = lines.map((line) => ({
    line,
    text: normalizeOutlineText(line.spans.map((span) => span.text).join("")),
  }))
    .filter((candidate) => candidate.text);
  return candidates.find((candidate) => candidate.text === normalizedTitle)?.line
    ?? candidates.find((candidate) => candidate.text.includes(normalizedTitle))?.line
    ?? candidates.find((candidate) => normalizedTitle.includes(candidate.text) && candidate.text.length >= 4)?.line;
}

function normalizeOutlineText(value: string) {
  return value.toLocaleLowerCase().replace(/[\s\p{P}\p{S}]+/gu, "");
}

function annotationColorTargets(annotationId: string) {
  const escapedId = window.CSS.escape(annotationId);
  return window.document.querySelectorAll<HTMLElement>(
    `.annotation-color-target[data-annotation-id="${escapedId}"]`,
  );
}
