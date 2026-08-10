import { useEffect, useRef, useState, type CSSProperties, type PointerEvent as ReactPointerEvent } from "react";
import type { Annotation, AnnotationRect, MarkupType } from "../../annotations";
import { copyText } from "../../../platform/clipboard";
import { usePdfSession } from "../hooks/usePdfSession";
import { usePdfViewportController } from "../hooks/usePdfViewportController";
import { PdfPage } from "./PdfPage";
import { PdfSelectionMenu, type PdfSelection } from "./PdfSelectionMenu";
import { PdfBookmarkMenu } from "./PdfBookmarkMenu";
import { PdfExistingBookmarkMenu } from "./PdfExistingBookmarkMenu";
import { PdfAnnotationMenu } from "./PdfAnnotationMenu";
import { PdfAreaAnnotationMenu } from "./PdfAreaAnnotationMenu";
import type { PdfNavigationTarget } from "../types/pdfNavigation";
import type { PdfBookmark, PdfBookmarkDraft } from "../types/pdfBookmark";
import type { PdfZoomMode } from "../state/PdfWorkspaceContext";
import type { PdfInteractionMode } from "../state/PdfWorkspaceContext";
import { readSelectionRects, resizeTextAnnotationAt, type TextAnnotationResizeEdge } from "../utils/selectionGeometry";
import type { PdfPageLink } from "../types/pdfStructure";
import { isAllowedExternalPdfLink } from "../utils/pdfLinks";
import { ExternalLinkConfirmationDialog } from "./ExternalLinkConfirmationDialog";

type PdfViewerProps = {
  scale: number;
  annotations: Annotation[];
  bookmarks: PdfBookmark[];
  zoomMode: PdfZoomMode;
  interactionMode: PdfInteractionMode;
  activeMarkupType: MarkupType;
  annotationColor: string;
  navigationTarget: PdfNavigationTarget | null;
  selectedAnnotationId: string | null;
  onCurrentPageChange: (page: number, pageY: number, pageX: number) => void;
  onFitScaleResolved: (scale: number) => void;
  onScaleChange: (scale: number) => void;
  onSelectionCapture: (selection: Omit<PdfSelection, "x" | "y" | "bookmarkX" | "bookmarkY">) => void;
  onClearSelection: () => void;
  onApplyMarkup: (type: MarkupType, selection: Omit<PdfSelection, "x" | "y" | "bookmarkX" | "bookmarkY">) => void;
  onAnnotationSelect: (id: string) => void;
  onAnnotationColor: (color: string) => void;
  onAnnotationColorPreview: (color: string) => void;
  onAnnotationConvert: (id: string, type: MarkupType) => void;
  onAnnotationDelete: (id: string) => void;
  onBookmarkAdd: (bookmark: PdfBookmarkDraft) => void;
  onBookmarkNavigate: (id: string) => void;
  onBookmarkUpdate: (id: string, changes: Partial<Pick<PdfBookmark, "title" | "color" | "page" | "x" | "y">>) => void;
  onBookmarkDelete: (id: string) => void;
  movingBookmarkId: string | null;
  onBeginBookmarkMove: (id: string | null) => void;
  transformingAreaId: string | null;
  onBeginAreaTransform: (id: string | null) => void;
  onAreaCreate: (page: number, rect: AnnotationRect) => void;
  onAnnotationUpdate: (annotation: Annotation) => void;
  onLinkNavigate: (page: number, pageX?: number, pageY?: number) => void;
  onExternalLinkOpen: (uri: string) => Promise<void>;
};

type ContextMenu =
  | ({ kind: "selection" } & PdfSelection)
  | { kind: "bookmark"; menuX: number; menuY: number; bookmark: PdfBookmarkDraft }
  | { kind: "existing-bookmark"; menuX: number; menuY: number; bookmarkId: string }
  | { kind: "annotation"; menuX: number; menuY: number; annotationId: string; bookmark: PdfBookmarkDraft };

type AreaTransformHandle = "move" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "nw";
const MINIMUM_AREA_WIDTH = 0.018;
const MINIMUM_AREA_HEIGHT = 0.014;

export function PdfReader(props: PdfViewerProps) {
  const { document } = usePdfSession();
  const { fitWidth, viewportHeight, renderCenterPage, scrollRef } = usePdfViewportController(
    document?.pageCount ?? 0, props.scale, props.navigationTarget, props.onCurrentPageChange, props.onScaleChange,
  );
  const [contextMenu, setContextMenu] = useState<ContextMenu | null>(null);
  const [activeSelection, setActiveSelection] = useState<{ page: number; rects: AnnotationRect[] } | null>(null);
  const [gestureActive, setGestureActive] = useState(false);
  const [bookmarkPreview, setBookmarkPreview] = useState<PdfBookmark | null>(null);
  const [areaPreview, setAreaPreview] = useState<{ page: number; rect: AnnotationRect } | null>(null);
  const [areaTransformPreview, setAreaTransformPreview] = useState<{ id: string; rect: AnnotationRect } | null>(null);
  const [textResizePreview, setTextResizePreview] = useState<{ id: string; text: string; rects: AnnotationRect[] } | null>(null);
  const [pendingExternalLink, setPendingExternalLink] = useState<string | null>(null);
  const [linkDestination, setLinkDestination] = useState<{ page: number; x?: number; y?: number; requestId: number } | null>(null);
  const bookmarkPreviewRef = useRef<PdfBookmark | null>(null);
  const bookmarkDraggedRef = useRef(false);
  const gestureRef = useRef<
    | { kind: "pan"; x: number; y: number; scrollLeft: number; scrollTop: number }
    | { kind: "eraser" }
    | { kind: "bookmark"; id: string; startX: number; startY: number }
    | { kind: "area-draw"; page: number; paper: DOMRect; startX: number; startY: number }
    | { kind: "area-transform"; id: string; handle: AreaTransformHandle; paper: HTMLElement;
        startX: number; startY: number; original: AnnotationRect }
    | { kind: "text-annotation-resize"; id: string; edge: TextAnnotationResizeEdge;
        paper: HTMLElement; originalRects: AnnotationRect[] }
    | null
  >(null);
  const erasedIdsRef = useRef(new Set<string>());
  const lastPointerRef = useRef<{ x: number; y: number } | null>(null);
  const selectionCaptureRef = useRef(props.onSelectionCapture);
  selectionCaptureRef.current = props.onSelectionCapture;

  useEffect(() => {
    if (props.zoomMode !== "fit-page" || fitWidth <= 0 || viewportHeight <= 0) return;
    const paper = scrollRef.current?.querySelector<HTMLElement>(`[data-page-number="${renderCenterPage}"] .pdf-paper`);
    const bounds = paper?.getBoundingClientRect();
    const aspectRatio = bounds && bounds.width > 0 ? bounds.height / bounds.width : 1.414;
    const availableHeight = Math.max(180, viewportHeight - 74);
    const nextScale = 1.5 * Math.min(1, availableHeight / (fitWidth * aspectRatio));
    props.onFitScaleResolved(Math.max(0.5, Math.min(2.5, nextScale)));
  }, [fitWidth, props.onFitScaleResolved, props.zoomMode, renderCenterPage, scrollRef, viewportHeight]);

  useEffect(() => {
    let frame = 0;
    const updateSelection = () => {
      window.cancelAnimationFrame(frame);
      frame = window.requestAnimationFrame(() => {
        const selection = window.getSelection();
        const anchor = selection?.anchorNode;
        const anchorElement = anchor instanceof Element ? anchor : anchor?.parentElement;
        const paper = anchorElement?.closest<HTMLElement>(".pdf-paper");
        const page = paper?.closest<HTMLElement>(".pdf-page");
        const pageNumber = Number(page?.dataset.pageNumber ?? 0);
        const rects = selection && paper ? readSelectionRects(selection, paper) : [];
        const text = selection?.toString().trim() ?? "";
        if (!pageNumber || !text || !rects.length) { setActiveSelection(null); return; }
        setActiveSelection({ page: pageNumber, rects });
        selectionCaptureRef.current({ page: pageNumber, text, rects });
      });
    };
    window.document.addEventListener("selectionchange", updateSelection);
    return () => {
      window.cancelAnimationFrame(frame);
      window.document.removeEventListener("selectionchange", updateSelection);
    };
  }, []);

  useEffect(() => {
    const clearInactiveSelection = (event: PointerEvent) => {
      if (event.button === 2) return;
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest(".pdf-annotation-tools, .selection-context-menu, .side-panel, .text-annotation-resize-layer")) return;
      window.getSelection()?.removeAllRanges();
      props.onClearSelection();
    };
    window.addEventListener("pointerdown", clearInactiveSelection);
    return () => window.removeEventListener("pointerdown", clearInactiveSelection);
  }, [props.onClearSelection]);

  useEffect(() => {
    if (!contextMenu) return;
    const close = () => setContextMenu(null);
    const closeOnEscape = (event: KeyboardEvent) => { if (event.key === "Escape") close(); };
    window.addEventListener("pointerdown", close);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", close);
      window.removeEventListener("keydown", closeOnEscape);
    };
  }, [contextMenu]);

  useEffect(() => {
    if (!linkDestination) return;
    const clearDestination = () => setLinkDestination(null);
    window.addEventListener("pointerdown", clearDestination, { once: true });
    return () => window.removeEventListener("pointerdown", clearDestination);
  }, [linkDestination]);

  useEffect(() => setLinkDestination(null), [document?.sessionId]);

  useEffect(() => {
    if (!props.transformingAreaId && !props.movingBookmarkId) return;
    const closeTransformModeOutsideTarget = (event: PointerEvent) => {
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest(".selection-context-menu")) return;
      if (props.transformingAreaId && target?.closest(`[data-area-id="${window.CSS.escape(props.transformingAreaId)}"]`)) return;
      if (props.movingBookmarkId && target?.closest(`[data-bookmark-id="${window.CSS.escape(props.movingBookmarkId)}"]`)) return;
      if (props.transformingAreaId) props.onBeginAreaTransform(null);
      if (props.movingBookmarkId) props.onBeginBookmarkMove(null);
    };
    window.addEventListener("pointerdown", closeTransformModeOutsideTarget);
    return () => window.removeEventListener("pointerdown", closeTransformModeOutsideTarget);
  }, [props.movingBookmarkId, props.onBeginAreaTransform, props.onBeginBookmarkMove, props.transformingAreaId]);

  if (!document) return <div className="reader-placeholder">正在加载 PDF...</div>;
  if (document.needsPassword) return <div className="reader-placeholder">该 PDF 需要密码。</div>;
  function handlePointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    if (event.button !== 0) return;
    lastPointerRef.current = { x: event.clientX, y: event.clientY };
    const target = event.target instanceof Element ? event.target : null;
    const textResizeHandle = target?.closest<HTMLElement>("[data-annotation-resize-edge]");
    const resizedAnnotation = props.annotations.find((annotation) => annotation.id === textResizeHandle?.dataset.annotationResizeId);
    const resizePaper = textResizeHandle?.closest<HTMLElement>(".pdf-paper");
    if (textResizeHandle && resizePaper && resizedAnnotation?.type !== "area"
      && resizedAnnotation?.type !== "note" && resizedAnnotation?.rects?.length) {
      event.preventDefault();
      event.stopPropagation();
      window.getSelection()?.removeAllRanges();
      gestureRef.current = {
        kind: "text-annotation-resize",
        id: resizedAnnotation.id,
        edge: textResizeHandle.dataset.annotationResizeEdge as TextAnnotationResizeEdge,
        paper: resizePaper,
        originalRects: resizedAnnotation.rects,
      };
      setTextResizePreview({ id: resizedAnnotation.id, text: resizedAnnotation.selectedText, rects: resizedAnnotation.rects });
      event.currentTarget.setPointerCapture(event.pointerId);
      setGestureActive(true);
      return;
    }
    const marker = target?.closest<HTMLElement>("[data-bookmark-id]");
    const draggedBookmark = props.bookmarks.find((bookmark) => bookmark.id === marker?.dataset.bookmarkId);
    if (draggedBookmark && props.movingBookmarkId === draggedBookmark.id) {
      bookmarkDraggedRef.current = false;
      bookmarkPreviewRef.current = draggedBookmark;
      setBookmarkPreview(draggedBookmark);
      gestureRef.current = { kind: "bookmark", id: draggedBookmark.id, startX: event.clientX, startY: event.clientY };
      event.currentTarget.setPointerCapture(event.pointerId);
      setGestureActive(true);
      event.preventDefault();
      event.stopPropagation();
      return;
    }
    const areaAnnotation = props.annotations.find((annotation) => annotation.id === props.transformingAreaId && annotation.type === "area");
    if (areaAnnotation?.rects?.[0]) {
      const paper = target?.closest<HTMLElement>(".pdf-paper");
      const handleElement = target?.closest<HTMLElement>("[data-area-handle]");
      const areaElement = target?.closest<HTMLElement>(`[data-area-id="${areaAnnotation.id}"]`);
      if (paper && (handleElement || areaElement)) {
        event.preventDefault();
        window.getSelection()?.removeAllRanges();
        gestureRef.current = { kind: "area-transform", id: areaAnnotation.id,
          handle: (handleElement?.dataset.areaHandle as AreaTransformHandle | undefined) ?? "move",
          paper,
          startX: (event.clientX - paper.getBoundingClientRect().left) / paper.getBoundingClientRect().width,
          startY: (event.clientY - paper.getBoundingClientRect().top) / paper.getBoundingClientRect().height,
          original: areaAnnotation.rects[0] };
        setAreaTransformPreview({ id: areaAnnotation.id, rect: areaAnnotation.rects[0] });
        event.currentTarget.setPointerCapture(event.pointerId);
        setGestureActive(true);
        event.stopPropagation();
        return;
      }
      props.onBeginAreaTransform(null);
    }
    if (props.movingBookmarkId) props.onBeginBookmarkMove(null);
    if (props.interactionMode === "area") {
      const paper = target?.closest<HTMLElement>(".pdf-paper");
      const page = paper?.closest<HTMLElement>(".pdf-page");
      const pageNumber = Number(page?.dataset.pageNumber ?? 0);
      if (!paper || !pageNumber) return;
      const bounds = paper.getBoundingClientRect();
      const startX = clamp01((event.clientX - bounds.left) / bounds.width);
      const startY = clamp01((event.clientY - bounds.top) / bounds.height);
      event.preventDefault();
      window.getSelection()?.removeAllRanges();
      gestureRef.current = { kind: "area-draw", page: pageNumber, paper: bounds, startX, startY };
      setAreaPreview({ page: pageNumber, rect: normalizedAreaRect(startX, startY, startX, startY) });
      event.currentTarget.setPointerCapture(event.pointerId);
      setGestureActive(true);
      return;
    }
    if (props.interactionMode === "pan") {
      event.preventDefault();
      window.getSelection()?.removeAllRanges();
      gestureRef.current = {
        kind: "pan", x: event.clientX, y: event.clientY,
        scrollLeft: event.currentTarget.scrollLeft, scrollTop: event.currentTarget.scrollTop,
      };
      event.currentTarget.setPointerCapture(event.pointerId);
      setGestureActive(true);
      return;
    }
    if (props.interactionMode === "eraser") {
      event.preventDefault();
      window.getSelection()?.removeAllRanges();
      erasedIdsRef.current.clear();
      gestureRef.current = { kind: "eraser" };
      event.currentTarget.setPointerCapture(event.pointerId);
      eraseAnnotationsAt(event.clientX, event.clientY);
      setGestureActive(true);
    }
  }

  function handlePointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    lastPointerRef.current = { x: event.clientX, y: event.clientY };
    const gesture = gestureRef.current;
    if (!gesture) return;
    event.preventDefault();
    if (gesture.kind === "pan") {
      event.currentTarget.scrollLeft = gesture.scrollLeft - (event.clientX - gesture.x);
      event.currentTarget.scrollTop = gesture.scrollTop - (event.clientY - gesture.y);
    } else if (gesture.kind === "eraser") {
      eraseAnnotationsAt(event.clientX, event.clientY);
    } else if (gesture.kind === "bookmark") {
      if (Math.hypot(event.clientX - gesture.startX, event.clientY - gesture.startY) > 3) bookmarkDraggedRef.current = true;
      const location = bookmarkLocationAt(event.clientX, event.clientY);
      const original = props.bookmarks.find((bookmark) => bookmark.id === gesture.id);
      if (!location || !original) return;
      const preview = { ...original, ...location };
      bookmarkPreviewRef.current = preview;
      setBookmarkPreview(preview);
    } else if (gesture.kind === "area-draw") {
      const x = clamp01((event.clientX - gesture.paper.left) / gesture.paper.width);
      const y = clamp01((event.clientY - gesture.paper.top) / gesture.paper.height);
      setAreaPreview({ page: gesture.page, rect: normalizedAreaRect(gesture.startX, gesture.startY, x, y) });
    } else if (gesture.kind === "area-transform") {
      updateAreaTransformPreview(gesture, event.clientX, event.clientY);
    } else if (gesture.kind === "text-annotation-resize") {
      updateTextAnnotationResizePreview(gesture, event.clientX, event.clientY);
    }
  }

  function updateTextAnnotationResizePreview(
    gesture: Extract<NonNullable<typeof gestureRef.current>, { kind: "text-annotation-resize" }>,
    clientX: number,
    clientY: number,
  ) {
    const preview = resizeTextAnnotationAt(
      gesture.paper,
      gesture.originalRects,
      gesture.edge,
      clientX,
      clientY,
    );
    if (preview) setTextResizePreview({ id: gesture.id, ...preview });
  }

  function updateAreaTransformPreview(gesture: Extract<NonNullable<typeof gestureRef.current>, { kind: "area-transform" }>, clientX: number, clientY: number) {
    const bounds = gesture.paper.getBoundingClientRect();
    const currentX = (clientX - bounds.left) / bounds.width;
    const currentY = (clientY - bounds.top) / bounds.height;
    setAreaTransformPreview({ id: gesture.id,
      rect: transformAreaRect(gesture.original, gesture.handle, currentX - gesture.startX, currentY - gesture.startY) });
  }

  function handleScrollDuringGesture() {
    const pointer = lastPointerRef.current;
    const gesture = gestureRef.current;
    if (!pointer || !gesture) return;
    window.requestAnimationFrame(() => {
      if (gestureRef.current !== gesture) return;
      if (gesture.kind === "area-transform") updateAreaTransformPreview(gesture, pointer.x, pointer.y);
      if (gesture.kind === "text-annotation-resize") updateTextAnnotationResizePreview(gesture, pointer.x, pointer.y);
      if (gesture.kind === "bookmark") {
        const location = bookmarkLocationAt(pointer.x, pointer.y);
        const original = props.bookmarks.find((bookmark) => bookmark.id === gesture.id);
        if (!location || !original) return;
        const preview = { ...original, ...location };
        bookmarkPreviewRef.current = preview;
        setBookmarkPreview(preview);
      }
    });
  }

  function handlePointerEnd(event: ReactPointerEvent<HTMLDivElement>) {
    if (gestureRef.current) {
      const gesture = gestureRef.current;
      if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
      gestureRef.current = null;
      setGestureActive(false);
      if (gesture.kind === "bookmark") {
        const preview = bookmarkPreviewRef.current;
        if (event.type !== "pointercancel" && bookmarkDraggedRef.current && preview) {
          props.onBookmarkUpdate(gesture.id, { page: preview.page, x: preview.x, y: preview.y });
        }
        bookmarkPreviewRef.current = null;
        setBookmarkPreview(null);
      } else if (gesture.kind === "area-draw") {
        const preview = areaPreview;
        setAreaPreview(null);
        if (event.type !== "pointercancel" && preview) props.onAreaCreate(preview.page, preview.rect);
      } else if (gesture.kind === "area-transform") {
        const preview = areaTransformPreview;
        const annotation = props.annotations.find((item) => item.id === gesture.id);
        setAreaTransformPreview(null);
        if (event.type !== "pointercancel" && preview && annotation) {
          props.onAnnotationUpdate({ ...annotation, rects: [preview.rect], updatedAt: new Date().toISOString() });
        }
      } else if (gesture.kind === "text-annotation-resize") {
        const preview = textResizePreview;
        const annotation = props.annotations.find((item) => item.id === gesture.id);
        setTextResizePreview(null);
        if (event.type !== "pointercancel" && preview && annotation) {
          props.onAnnotationUpdate({
            ...annotation,
            selectedText: preview.text,
            rects: preview.rects,
            updatedAt: new Date().toISOString(),
          });
        }
      }
      return;
    }
    if (props.interactionMode !== "markup" || event.button !== 0) return;
    window.requestAnimationFrame(() => {
      const selection = window.getSelection();
      const anchor = selection?.anchorNode;
      const anchorElement = anchor instanceof Element ? anchor : anchor?.parentElement;
      const paper = anchorElement?.closest<HTMLElement>(".pdf-paper");
      const page = paper?.closest<HTMLElement>(".pdf-page");
      const pageNumber = Number(page?.dataset.pageNumber ?? 0);
      const text = selection?.toString().trim() ?? "";
      const rects = selection && paper ? readSelectionRects(selection, paper) : [];
      if (!pageNumber || !text || !rects.length) return;
      props.onApplyMarkup(props.activeMarkupType, { page: pageNumber, text, rects });
    });
  }

  function eraseAnnotationsAt(clientX: number, clientY: number) {
    const target = window.document.elementFromPoint(clientX, clientY);
    const paper = target?.closest<HTMLElement>(".pdf-paper");
    const page = paper?.closest<HTMLElement>(".pdf-page");
    const pageNumber = Number(page?.dataset.pageNumber ?? 0);
    if (!paper || !pageNumber) return;
    const bounds = paper.getBoundingClientRect();
    const x = (clientX - bounds.left) / bounds.width;
    const y = (clientY - bounds.top) / bounds.height;
    const toleranceX = 9 / bounds.width;
    const toleranceY = 9 / bounds.height;
    props.annotations.filter((annotation) => annotation.page === pageNumber && annotation.rects?.some((rect) => (
      x >= rect.left - toleranceX && x <= rect.left + rect.width + toleranceX
      && y >= rect.top - toleranceY && y <= rect.top + rect.height + toleranceY
    ))).forEach((annotation) => {
      if (erasedIdsRef.current.has(annotation.id)) return;
      erasedIdsRef.current.add(annotation.id);
      props.onAnnotationDelete(annotation.id);
    });
  }

  function bookmarkLocationAt(clientX: number, clientY: number) {
    const target = window.document.elementFromPoint(clientX, clientY);
    const paper = target?.closest<HTMLElement>(".pdf-paper");
    const page = paper?.closest<HTMLElement>(".pdf-page");
    const pageNumber = Number(page?.dataset.pageNumber ?? 0);
    if (!paper || !pageNumber) return null;
    const bounds = paper.getBoundingClientRect();
    return {
      page: pageNumber,
      x: Math.max(0, Math.min(1, (clientX - bounds.left) / bounds.width)),
      y: Math.max(0, Math.min(1, (clientY - bounds.top) / bounds.height)),
    };
  }

  function activateLink(link: PdfPageLink) {
    if (link.targetPage) {
      setLinkDestination({ page: link.targetPage, x: link.targetX, y: link.targetY, requestId: Date.now() });
      props.onLinkNavigate(link.targetPage, link.targetX, link.targetY);
      return;
    }
    if (isAllowedExternalPdfLink(link.uri)) setPendingExternalLink(link.uri.trim());
  }

  const readerStyle = {
    "--pdf-markup-cursor": buildMarkupCursor(props.activeMarkupType, props.annotationColor),
  } as CSSProperties;

  return <div className={`pdf-scroll interaction-${props.interactionMode} ${gestureActive ? "gesture-active" : ""} ${props.movingBookmarkId ? "bookmark-move-active" : ""} ${bookmarkPreview ? "bookmark-drag-active" : ""} ${props.transformingAreaId ? "area-transform-active" : ""} ${areaTransformPreview ? "area-transform-drag-active" : ""} ${textResizePreview ? "text-annotation-resize-active" : ""}`}
    style={readerStyle} ref={scrollRef} onPointerDown={handlePointerDown} onPointerMove={handlePointerMove}
    onPointerUp={handlePointerEnd} onPointerCancel={handlePointerEnd} onScroll={handleScrollDuringGesture}>
    {Array.from({ length: document.pageCount }, (_, index) => (
      <PdfPage key={index + 1} sessionId={document.sessionId} pageNumber={index + 1} scale={props.scale} fitWidth={fitWidth}
        annotations={props.annotations.filter((annotation) => annotation.page === index + 1)}
        bookmarks={bookmarksForPage(props.bookmarks, bookmarkPreview, index + 1)}
        active={Math.abs(index + 1 - renderCenterPage) <= 2}
        activeSelectionRects={activeSelection?.page === index + 1 ? activeSelection.rects : []}
        selectedAnnotationId={props.selectedAnnotationId}
        movingBookmarkId={props.movingBookmarkId}
        transformingAreaId={props.transformingAreaId}
        areaPreview={areaPreview?.page === index + 1 ? areaPreview.rect : null}
        areaTransformPreview={areaTransformPreview}
        textResizePreview={textResizePreview}
        onAnnotationSelect={props.onAnnotationSelect}
        onAreaTransform={(id) => props.onBeginAreaTransform(id)}
        onBookmarkNavigate={(id) => {
          if (bookmarkDraggedRef.current) { bookmarkDraggedRef.current = false; return; }
          props.onBookmarkNavigate(id);
        }}
        onBookmarkMove={(id) => props.onBeginBookmarkMove(id)}
        onExistingBookmarkContextMenu={(event, bookmark) => {
          const point = clampMenuPoint(event.clientX, event.clientY, 210, 245);
          setContextMenu({ kind: "existing-bookmark", menuX: point.x, menuY: point.y, bookmarkId: bookmark.id });
        }}
        onBookmarkContextMenu={(event, bookmark) => {
          const point = clampMenuPoint(event.clientX, event.clientY, 190, 58);
          setContextMenu({ kind: "bookmark", menuX: point.x, menuY: point.y, bookmark });
        }}
        onAnnotationContextMenu={(event, annotation, bookmark) => {
          props.onAnnotationSelect(annotation.id);
          const point = clampMenuPoint(event.clientX, event.clientY, annotation.type === "area" ? 320 : 286, annotation.type === "area" ? 620 : 355);
          setContextMenu({ kind: "annotation", menuX: point.x, menuY: point.y, annotationId: annotation.id, bookmark });
        }}
        onTextContextMenu={(event, text, rects, bookmark) => {
          const point = clampMenuPoint(event.clientX, event.clientY, 230, 286);
          setContextMenu({ kind: "selection", x: point.x, y: point.y,
            page: index + 1, text, rects, bookmarkX: bookmark.x, bookmarkY: bookmark.y });
        }} onLinkActivate={activateLink}
        linkDestination={linkDestination?.page === index + 1 ? linkDestination : null} />
    ))}
    {contextMenu?.kind === "selection" && <PdfSelectionMenu selection={contextMenu}
      onCopy={(selection) => { void copyText(selection.text); setContextMenu(null); }}
      onApply={(type, selection) => {
        props.onApplyMarkup(type, selection);
        setContextMenu(null);
        window.getSelection()?.removeAllRanges();
      }}
      onAddBookmark={(selection) => {
        setContextMenu({ kind: "bookmark", menuX: selection.x, menuY: selection.y,
          bookmark: { page: selection.page, x: selection.bookmarkX, y: selection.bookmarkY } });
      }} />}
    {contextMenu?.kind === "bookmark" && <PdfBookmarkMenu menuX={contextMenu.menuX} menuY={contextMenu.menuY}
      bookmark={contextMenu.bookmark} onAdd={(bookmark) => {
      props.onBookmarkAdd(bookmark);
      setContextMenu(null);
    }} />}
    {contextMenu?.kind === "existing-bookmark" && (() => {
      const bookmark = props.bookmarks.find((item) => item.id === contextMenu.bookmarkId);
      return bookmark ? <PdfExistingBookmarkMenu x={contextMenu.menuX} y={contextMenu.menuY} bookmark={bookmark}
        onUpdate={(changes) => props.onBookmarkUpdate(bookmark.id, changes)}
        onMove={() => { props.onBeginBookmarkMove(bookmark.id); setContextMenu(null); }}
        onDelete={() => { props.onBookmarkDelete(bookmark.id); setContextMenu(null); }}
        onClose={() => setContextMenu(null)} /> : null;
    })()}
    {contextMenu?.kind === "annotation" && (() => {
      const annotation = props.annotations.find((item) => item.id === contextMenu.annotationId);
      if (annotation?.type === "area") return <PdfAreaAnnotationMenu x={contextMenu.menuX} y={contextMenu.menuY} annotation={annotation}
        onUpdate={props.onAnnotationUpdate}
        onMove={() => { props.onBeginAreaTransform(annotation.id); setContextMenu(null); }}
        onDelete={() => { props.onAnnotationDelete(annotation.id); setContextMenu(null); }} />;
      return annotation ? <PdfAnnotationMenu x={contextMenu.menuX} y={contextMenu.menuY} annotation={annotation}
        onCopy={() => { void copyText(annotation.selectedText); setContextMenu(null); }}
        onColor={props.onAnnotationColor}
        onConvert={(type) => { props.onAnnotationConvert(annotation.id, type); setContextMenu(null); }}
        onDelete={() => { props.onAnnotationDelete(annotation.id); setContextMenu(null); }}
        onAddBookmark={() => { setContextMenu({ kind: "bookmark", menuX: contextMenu.menuX, menuY: contextMenu.menuY,
          bookmark: contextMenu.bookmark }); }} /> : null;
    })()}
    {pendingExternalLink && <ExternalLinkConfirmationDialog uri={pendingExternalLink}
      onCancel={() => setPendingExternalLink(null)} onConfirm={() => {
        const uri = pendingExternalLink;
        setPendingExternalLink(null);
        void props.onExternalLinkOpen(uri);
      }} />}
  </div>;
}

function bookmarksForPage(bookmarks: PdfBookmark[], preview: PdfBookmark | null, page: number) {
  const stable = bookmarks.filter((bookmark) => bookmark.page === page && bookmark.id !== preview?.id);
  return preview?.page === page ? [...stable, preview] : stable;
}

function normalizedAreaRect(startX: number, startY: number, endX: number, endY: number): AnnotationRect {
  const drawingRight = endX >= startX;
  const drawingDown = endY >= startY;
  let width = Math.max(Math.abs(endX - startX), MINIMUM_AREA_WIDTH);
  let height = Math.max(Math.abs(endY - startY), MINIMUM_AREA_HEIGHT);
  let left = drawingRight ? startX : startX - width;
  let top = drawingDown ? startY : startY - height;
  left = Math.max(0, Math.min(left, 1 - MINIMUM_AREA_WIDTH));
  top = Math.max(0, Math.min(top, 1 - MINIMUM_AREA_HEIGHT));
  width = Math.min(width, 1 - left);
  height = Math.min(height, 1 - top);
  return { left, top, width, height };
}

function transformAreaRect(original: AnnotationRect, handle: AreaTransformHandle, deltaX: number, deltaY: number): AnnotationRect {
  if (handle === "move") {
    return { ...original,
      left: Math.max(0, Math.min(1 - original.width, original.left + deltaX)),
      top: Math.max(0, Math.min(1 - original.height, original.top + deltaY)) };
  }
  let left = original.left;
  let top = original.top;
  let right = original.left + original.width;
  let bottom = original.top + original.height;
  if (handle.includes("w")) left = Math.max(0, Math.min(right - MINIMUM_AREA_WIDTH, left + deltaX));
  if (handle.includes("e")) right = Math.min(1, Math.max(left + MINIMUM_AREA_WIDTH, right + deltaX));
  if (handle.includes("n")) top = Math.max(0, Math.min(bottom - MINIMUM_AREA_HEIGHT, top + deltaY));
  if (handle.includes("s")) bottom = Math.min(1, Math.max(top + MINIMUM_AREA_HEIGHT, bottom + deltaY));
  return { left, top, width: right - left, height: bottom - top };
}

function clamp01(value: number) {
  return Math.max(0, Math.min(1, value));
}

function clampMenuPoint(x: number, y: number, width: number, height: number) {
  const margin = 8;
  return {
    x: Math.max(margin, Math.min(x, window.innerWidth - width - margin)),
    y: Math.max(margin, Math.min(y, window.innerHeight - height - margin)),
  };
}

function buildMarkupCursor(type: MarkupType, requestedColor: string) {
  const color = /^#(?:[0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(requestedColor) ? requestedColor : "#176749";
  const pen = `<path d="M7 22 15.7 6.8a2 2 0 0 1 2.8-.7l4.3 2.5a2 2 0 0 1 .7 2.8L14.8 26H8z" fill="${color}" stroke="#34404c" stroke-width="2" stroke-linejoin="round"/><path d="m16.2 7.2 6.7 3.9" fill="none" stroke="#34404c" stroke-width="2"/>`;
  const mark = type === "underline"
    ? `<path d="M4 27h22" fill="none" stroke="${color}" stroke-width="2.5" stroke-linecap="round"/>`
    : type === "squiggly"
      ? `<path d="M3 27c2.1-2.6 4.2-2.6 6.3 0s4.2 2.6 6.3 0 4.2-2.6 6.3 0 4.2 2.6 6.3 0" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round"/>`
      : type === "strikeout"
        ? `<path d="M3 17h24" fill="none" stroke="${color}" stroke-width="2.5" stroke-linecap="round"/>`
        : `<path d="M3 27h24" fill="none" stroke="${color}" stroke-width="4" stroke-linecap="round" opacity=".82"/>`;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 30 30">${pen}${mark}</svg>`;
  const hotspotY = type === "strikeout" ? 17 : 27;
  return `url("data:image/svg+xml,${encodeURIComponent(svg)}") 3 ${hotspotY}`;
}
