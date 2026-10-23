import { useEffect, useRef, useState, type CSSProperties } from "react";
import { Bookmark, MessageSquareText } from "lucide-react";
import type { Annotation, AnnotationRect } from "../../annotations";
import { mupdfRepository } from "../repositories/mupdfRepository";
import type { PdfPageStructure } from "../types/pdfStructure";
import type { PdfPageLink } from "../types/pdfStructure";
import { mergeSelectionRects, readSelectionRects } from "../utils/selectionGeometry";
import { PdfTextLayer } from "./PdfTextLayer";
import { PdfSearchOverlay } from "../search/components/PdfSearchOverlay";
import type { PdfBookmark, PdfBookmarkDraft } from "../types/pdfBookmark";
import { isAllowedExternalPdfLink, pdfLinkTooltip } from "../utils/pdfLinks";

type PdfPageProps = {
  sessionId: string;
  pageNumber: number;
  scale: number;
  fitWidth: number;
  annotations: Annotation[];
  bookmarks: PdfBookmark[];
  active: boolean;
  activeSelectionRects: AnnotationRect[];
  selectedAnnotationId: string | null;
  movingBookmarkId: string | null;
  transformingAreaId: string | null;
  areaPreview: AnnotationRect | null;
  areaTransformPreview: { id: string; rect: AnnotationRect } | null;
  textResizePreview: { id: string; rects: AnnotationRect[] } | null;
  onAnnotationSelect: (id: string) => void;
  onAreaTransform: (id: string) => void;
  onBookmarkNavigate: (id: string) => void;
  onBookmarkMove: (id: string) => void;
  onExistingBookmarkContextMenu: (event: React.MouseEvent, bookmark: PdfBookmark) => void;
  onBookmarkContextMenu: (event: React.MouseEvent, bookmark: PdfBookmarkDraft) => void;
  onAnnotationContextMenu: (event: React.MouseEvent, annotation: Annotation, bookmark: PdfBookmarkDraft) => void;
  onTextContextMenu: (event: React.MouseEvent, text: string, rects: AnnotationRect[], bookmark: PdfBookmarkDraft) => void;
  onLinkActivate: (link: PdfPageLink) => void;
  linkDestination?: { x?: number; y?: number; requestId: number } | null;
};

export function PdfPage({ sessionId, pageNumber, scale, fitWidth, annotations, bookmarks, active, activeSelectionRects, selectedAnnotationId, movingBookmarkId,
  transformingAreaId, areaPreview, areaTransformPreview, textResizePreview,
  onAnnotationSelect, onAreaTransform, onBookmarkNavigate, onBookmarkMove, onExistingBookmarkContextMenu, onBookmarkContextMenu, onAnnotationContextMenu, onTextContextMenu,
  onLinkActivate, linkDestination }: PdfPageProps) {
  const sectionRef = useRef<HTMLElement | null>(null);
  const paperRef = useRef<HTMLDivElement | null>(null);
  const [structure, setStructure] = useState<PdfPageStructure | null>(null);
  const [pageSize, setPageSize] = useState<{ width: number; height: number } | null>(null);
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [imageLoaded, setImageLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hasRenderedImageRef = useRef(false);
  const imageRenderScaleRef = useRef(1);
  const metrics = structure ?? pageSize;
  const displayScale = metrics && fitWidth > 0 ? (fitWidth / metrics.width) * (scale / 1.5) : 1;
  const placeholderWidth = fitWidth > 0 ? fitWidth * (scale / 1.5) : 600;
  const displaySize = metrics
    ? { width: metrics.width * displayScale, height: metrics.height * displayScale }
    : { width: placeholderWidth, height: placeholderWidth * 1.414 };

  useEffect(() => {
    if (!active) { setStructure(null); setPageSize(null); setError(null); return; }
    let cancelled = false;
    setStructure(null);
    setPageSize(null);
    setImageUrl(null);
    setImageLoaded(false);
    hasRenderedImageRef.current = false;
    setError(null);
    const timer = window.setTimeout(() => {
      mupdfRepository.pageStructure(sessionId, pageNumber - 1)
        .then((page) => { if (!cancelled) { setStructure(page); setPageSize({ width: page.width, height: page.height }); } })
        .catch((reason) => { if (!cancelled) setError(String(reason)); });
    }, 16);
    return () => { cancelled = true; window.clearTimeout(timer); };
  }, [active, pageNumber, sessionId]);

  useEffect(() => {
    if (!active || fitWidth <= 0) {
      setImageUrl(null);
      hasRenderedImageRef.current = false;
      return;
    }
    let cancelled = false;
    let url: string | null = null;
    let timer: number | null = null;
    const pixelRatio = Math.min(window.devicePixelRatio || 1, 2);
    const renderScale = (metrics ? displayScale : Math.max(0.5, scale / 1.5)) * pixelRatio;
    const requestImage = () => {
      mupdfRepository.renderPage(sessionId, pageNumber - 1, renderScale)
        .then((bytes) => {
          if (cancelled) return;
          url = URL.createObjectURL(new Blob([new Uint8Array(bytes).buffer], { type: "image/png" }));
          imageRenderScaleRef.current = renderScale;
          hasRenderedImageRef.current = true;
          setImageLoaded(false);
          setImageUrl(url);
        })
        .catch((reason) => { if (!cancelled) setError(String(reason)); });
    };
    if (hasRenderedImageRef.current) timer = window.setTimeout(requestImage, 80);
    else requestImage();
    return () => {
      cancelled = true;
      if (timer !== null) window.clearTimeout(timer);
      if (url) URL.revokeObjectURL(url);
    };
  }, [active, displayScale, fitWidth, metrics, pageNumber, scale, sessionId]);

  const selectedAnnotation = annotations.find((annotation) => annotation.id === selectedAnnotationId);
  const selectedRects = selectedAnnotation && textResizePreview?.id === selectedAnnotation.id
    ? textResizePreview.rects
    : selectedAnnotation?.rects;
  const selectedHitArea = selectedAnnotation && selectedAnnotation.type !== "area"
    ? annotationHitArea({ ...selectedAnnotation, rects: selectedRects })
    : null;
  const selectedResizeRects = selectedAnnotation && selectedAnnotation.type !== "area"
    && selectedAnnotation.type !== "note" && selectedRects?.length
    ? mergeSavedRects(selectedRects, displaySize.width, displaySize.height)
    : [];
  const transformedArea = annotations.find((annotation) => annotation.id === transformingAreaId && annotation.type === "area");
  const transformedRect = transformedArea
    ? (areaTransformPreview?.id === transformedArea.id ? areaTransformPreview.rect : transformedArea.rects?.[0])
    : null;
  const destinationHighlight = structure && linkDestination
    ? resolveLinkDestinationHighlight(structure, linkDestination.x, linkDestination.y)
    : null;
  const pageVisible = active && Boolean(imageUrl) && imageLoaded;

  return (
    <section ref={sectionRef} className="pdf-page" style={{ width: displaySize.width }} data-page-number={pageNumber}>
      <div className="pdf-paper" ref={paperRef} style={{ width: displaySize.width, height: displaySize.height }}
        onClick={(event) => {
          if (!paperRef.current || !window.getSelection()?.isCollapsed) return;
          const annotation = hitTestAnnotation(event, paperRef.current, annotations);
          if (annotation) onAnnotationSelect(annotation.id);
        }}
        onDoubleClick={(event) => {
          if (!paperRef.current) return;
          const annotation = hitTestAnnotation(event, paperRef.current, annotations);
          if (annotation?.type !== "area") return;
          event.preventDefault();
          event.stopPropagation();
          onAnnotationSelect(annotation.id);
          onAreaTransform(annotation.id);
        }}
        onContextMenu={(event) => handleContextMenu(event, paperRef.current, pageNumber, annotations,
          onTextContextMenu, onAnnotationContextMenu, onBookmarkContextMenu)}>
        {active && imageUrl && <img className="pdf-page-image" src={imageUrl} alt={`第 ${pageNumber} 页`} draggable={false}
          onLoad={(event) => {
            setImageLoaded(true);
            if (!pageSize && imageRenderScaleRef.current > 0) {
              setPageSize({ width: event.currentTarget.naturalWidth / imageRenderScaleRef.current,
                height: event.currentTarget.naturalHeight / imageRenderScaleRef.current });
            }
          }} />}
        {active && !imageUrl && <div className="pdf-page-loading">{error ? `页面加载失败：${error}` : "正在渲染…"}</div>}
        <div className="active-selection-layer" aria-hidden="true">
          {pageVisible && activeSelectionRects.map((rect, index) => (
            <span key={index} style={{
              left: `${rect.left * 100}%`, top: `${rect.top * 100}%`,
              width: `${rect.width * 100}%`, height: `${rect.height * 100}%`,
            }} />
          ))}
        </div>
        {pageVisible && <PdfSearchOverlay pageNumber={pageNumber} />}
        {pageVisible && destinationHighlight && linkDestination && <div className="pdf-link-destination-layer" aria-hidden="true">
          <span key={linkDestination.requestId} className="pdf-link-destination-highlight" style={{
            left: `${destinationHighlight.left * 100}%`, top: `${destinationHighlight.top * 100}%`,
            width: `${destinationHighlight.width * 100}%`, height: `${destinationHighlight.height * 100}%`,
          }} />
        </div>}
        <div className="annotation-geometry-layer" aria-hidden="true"
          style={{ "--annotation-visual-scale": Math.max(0.6, Math.sqrt(scale / 1.5)) } as CSSProperties}>
          {pageVisible && selectedHitArea && selectedAnnotation && (
            <span className="annotation-selection-outline annotation-color-target" data-annotation-id={selectedAnnotation.id} style={{
              left: `${selectedHitArea.left * 100}%`,
              top: `${selectedHitArea.top * 100}%`,
              width: `${(selectedHitArea.right - selectedHitArea.left) * 100}%`,
              height: `${(selectedHitArea.bottom - selectedHitArea.top) * 100}%`,
              color: `var(--annotation-color, ${selectedAnnotation.color})`,
            }} />
          )}
          {pageVisible && annotations.filter((annotation) => annotation.type === "area" && annotation.rects?.length).map((annotation) => {
            const rect = areaTransformPreview?.id === annotation.id ? areaTransformPreview.rect : annotation.rects![0];
            return <span className={`annotation-area annotation-color-target ${selectedAnnotationId === annotation.id ? "selected" : ""}`}
              data-annotation-id={annotation.id} key={annotation.id} style={{
                left: `${rect.left * 100}%`, top: `${rect.top * 100}%`, width: `${rect.width * 100}%`, height: `${rect.height * 100}%`,
                ...areaVisualStyle(annotation, scale),
              }} />;
          })}
          {pageVisible && annotations.filter((annotation) => annotation.type !== "area" && annotation.rects?.length).flatMap((annotation) => {
            const sourceRects = textResizePreview?.id === annotation.id ? textResizePreview.rects : annotation.rects!;
            const rects = mergeSavedRects(sourceRects, displaySize.width, displaySize.height);
            return rects.map((rect, index) => (
              <span className={`annotation-mark annotation-color-target annotation-${annotation.type} ${selectedAnnotationId === annotation.id ? "selected" : ""}`}
                data-annotation-id={annotation.id} key={`${annotation.id}-${index}`} style={{
                  left: `${rect.left * 100}%`, top: `${rect.top * 100}%`, width: `${rect.width * 100}%`,
                  height: `${rect.height * 100}%`, color: `var(--annotation-color, ${annotation.color})`,
                }} />
            ));
          })}
          {pageVisible && annotations.filter((annotation) => (annotation.hasNote || annotation.note.trim()) && annotation.rects?.length).map((annotation) => {
            const first = annotation.rects![0];
            return <span className={`annotation-note-marker annotation-color-target ${selectedAnnotationId === annotation.id ? "selected" : ""}`}
              data-annotation-id={annotation.id} key={`note-${annotation.id}`} style={{ left: `${first.left * 100}%`, top: `${first.top * 100}%`,
                background: `var(--annotation-color, ${annotation.color})` }}>
              <MessageSquareText size={10} />
            </span>;
          })}
        </div>
        {pageVisible && areaPreview && <div className="area-drawing-layer" aria-hidden="true"><span style={{
          left: `${areaPreview.left * 100}%`, top: `${areaPreview.top * 100}%`,
          width: `${areaPreview.width * 100}%`, height: `${areaPreview.height * 100}%`,
        }} /></div>}
        {pageVisible && transformedArea && transformedRect && <div className="area-transform-layer">
          <span className="area-transform-box" data-area-id={transformedArea.id} style={{
            left: `${transformedRect.left * 100}%`, top: `${transformedRect.top * 100}%`,
            width: `${transformedRect.width * 100}%`, height: `${transformedRect.height * 100}%`,
          }}>
            {(["nw", "n", "ne", "e", "se", "s", "sw", "w"] as const).map((handle) =>
              <i key={handle} className={`area-resize-handle handle-${handle}`} data-area-handle={handle} />)}
          </span>
        </div>}
        {pageVisible && structure && <PdfTextLayer structure={structure} displayScale={displayScale} />}
        {pageVisible && selectedAnnotation && selectedResizeRects.length > 0 && (
          <div className={`text-annotation-resize-layer ${selectedAnnotation.type === "highlight" ? "highlight-handles" : ""}`}>
            <button className="text-annotation-resize-handle handle-start" type="button"
              data-annotation-resize-id={selectedAnnotation.id} data-annotation-resize-edge="start"
              aria-label="调整标记起始位置" style={{
                left: `${selectedResizeRects[0].left * 100}%`,
                top: `${selectedResizeRects[0].top * 100}%`,
                height: `${selectedResizeRects[0].height * 100}%`,
                color: selectedAnnotation.color,
              }} />
            <button className="text-annotation-resize-handle handle-end" type="button"
              data-annotation-resize-id={selectedAnnotation.id} data-annotation-resize-edge="end"
              aria-label="调整标记结束位置" style={{
                left: `${(selectedResizeRects[selectedResizeRects.length - 1].left + selectedResizeRects[selectedResizeRects.length - 1].width) * 100}%`,
                top: `${selectedResizeRects[selectedResizeRects.length - 1].top * 100}%`,
                height: `${selectedResizeRects[selectedResizeRects.length - 1].height * 100}%`,
                color: selectedAnnotation.color,
              }} />
          </div>
        )}
        {pageVisible && structure && <div className="pdf-link-layer" aria-label={`第 ${pageNumber} 页链接`}>
          {structure.links.map((link, index) => {
            const allowed = Boolean(link.targetPage) || isAllowedExternalPdfLink(link.uri);
            return <button key={`${link.uri}-${index}`} className={`pdf-link-hotspot ${allowed ? "" : "blocked"}`}
              style={{ left: link.bounds.left * displayScale, top: link.bounds.top * displayScale,
                width: (link.bounds.right - link.bounds.left) * displayScale,
                height: (link.bounds.bottom - link.bounds.top) * displayScale }}
              onClick={(event) => { event.stopPropagation(); if (allowed) onLinkActivate(link); }}
              data-tooltip={pdfLinkTooltip(link)} aria-label={pdfLinkTooltip(link)} aria-disabled={!allowed} />;
          })}
        </div>}
        {pageVisible && bookmarks.map((bookmark) => (
          <button className={`pdf-bookmark-marker ${movingBookmarkId === bookmark.id ? "bookmark-moving" : ""}`} key={bookmark.id}
            style={{ left: `${bookmark.x * 100}%`, top: `${bookmark.y * 100}%`, background: bookmark.color }}
            data-bookmark-id={bookmark.id}
            onClick={(event) => { event.stopPropagation(); onBookmarkNavigate(bookmark.id); }}
            onDoubleClick={(event) => { event.preventDefault(); event.stopPropagation(); onBookmarkMove(bookmark.id); }}
            onContextMenu={(event) => { event.preventDefault(); event.stopPropagation(); onExistingBookmarkContextMenu(event, bookmark); }}
            data-tooltip={bookmark.title} aria-label={bookmark.title}>
            <Bookmark size={15} fill="currentColor" />
          </button>
        ))}
      </div>
      <div className="page-number">第 {pageNumber} 页</div>
    </section>
  );
}

function resolveLinkDestinationHighlight(structure: PdfPageStructure, x?: number, y?: number) {
  const lines = structure.blocks.flatMap((block) => block.lines);
  const targetX = (x ?? 0.05) * structure.width;
  const targetY = (y ?? 0) * structure.height;
  const line = lines.reduce<(typeof lines)[number] | null>((closest, candidate) => {
    if (!closest) return candidate;
    const score = linkLineDistance(candidate.bounds, targetX, targetY, x !== undefined);
    const closestScore = linkLineDistance(closest.bounds, targetX, targetY, x !== undefined);
    return score < closestScore ? candidate : closest;
  }, null);
  if (!line) {
    return { left: Math.max(0, Math.min(0.92, x ?? 0.05)), top: Math.max(0, Math.min(0.96, y ?? 0.02)), width: 0.42, height: 0.035 };
  }
  const paddingX = Math.min(0.012, 5 / structure.width);
  const paddingY = Math.min(0.008, 3 / structure.height);
  const left = Math.max(0, line.bounds.left / structure.width - paddingX);
  const top = Math.max(0, line.bounds.top / structure.height - paddingY);
  const right = Math.min(1, line.bounds.right / structure.width + paddingX);
  const bottom = Math.min(1, line.bounds.bottom / structure.height + paddingY);
  return { left, top, width: Math.max(0.02, right - left), height: Math.max(0.018, bottom - top) };
}

function areaVisualStyle(annotation: Annotation, scale: number): CSSProperties {
  const style = annotation.areaStyle;
  if (!style) return {};
  const visualScale = Math.max(0.6, Math.sqrt(scale / 1.5));
  const radius = style.radius * visualScale;
  const corners = style.roundedCorners;
  return {
    borderColor: annotation.color,
    borderStyle: style.borderStyle,
    borderWidth: style.borderStyle === "none" ? 0 : Math.max(1, style.borderWidth * visualScale),
    borderRadius: `${corners.topLeft ? radius : 0}px ${corners.topRight ? radius : 0}px ${corners.bottomRight ? radius : 0}px ${corners.bottomLeft ? radius : 0}px`,
    background: `color-mix(in srgb, ${style.backgroundColor} ${Math.round((style.opacity ?? 0.22) * 100)}%, transparent)`,
  };
}

function linkLineDistance(bounds: PdfPageStructure["blocks"][number]["lines"][number]["bounds"], x: number, y: number, useX: boolean) {
  const centerY = (bounds.top + bounds.bottom) / 2;
  const horizontalDistance = x < bounds.left ? bounds.left - x : x > bounds.right ? x - bounds.right : 0;
  return Math.abs(centerY - y) + (useX ? horizontalDistance * 0.2 : 0);
}

function handleContextMenu(event: React.MouseEvent, paper: HTMLDivElement | null, pageNumber: number, annotations: Annotation[],
  onTextContextMenu: PdfPageProps["onTextContextMenu"], onAnnotationContextMenu: PdfPageProps["onAnnotationContextMenu"],
  onBookmarkContextMenu: PdfPageProps["onBookmarkContextMenu"]) {
  const selection = window.getSelection();
  if (!paper) return;
  const bounds = paper.getBoundingClientRect();
  const bookmark = {
    page: pageNumber,
    x: Math.max(0, Math.min(1, (event.clientX - bounds.left) / bounds.width)),
    y: Math.max(0, Math.min(1, (event.clientY - bounds.top) / bounds.height)),
  };
  if (selection && !selection.isCollapsed && selection.toString().trim() && selection.rangeCount > 0) {
    const rects = readSelectionRects(selection, paper);
    if (rects.length) {
      event.preventDefault();
      onTextContextMenu(event, selection.toString().trim(), rects, bookmark);
      return;
    }
  }
  const annotation = hitTestAnnotation(event, paper, annotations);
  if (annotation) {
    event.preventDefault();
    onAnnotationContextMenu(event, annotation, bookmark);
    return;
  }
  event.preventDefault();
  onBookmarkContextMenu(event, bookmark);
}

function hitTestAnnotation(event: React.MouseEvent, paper: HTMLDivElement, annotations: Annotation[]) {
  const bounds = paper.getBoundingClientRect();
  const x = (event.clientX - bounds.left) / bounds.width;
  const y = (event.clientY - bounds.top) / bounds.height;
  const toleranceX = 5 / bounds.width;
  const toleranceY = 5 / bounds.height;

  return annotations.find((annotation) => {
    const hitArea = annotationHitArea(annotation);
    if (!hitArea) return false;
    return x >= hitArea.left - toleranceX
      && x <= hitArea.right + toleranceX
      && y >= hitArea.top - toleranceY
      && y <= hitArea.bottom + toleranceY;
  });
}

function annotationHitArea(annotation: Annotation) {
  if (!annotation.rects?.length) return null;
  return annotation.rects.reduce(
    (area, rect) => ({
      left: Math.min(area.left, rect.left),
      top: Math.min(area.top, rect.top),
      right: Math.max(area.right, rect.left + rect.width),
      bottom: Math.max(area.bottom, rect.top + rect.height),
    }),
    { left: 1, top: 1, right: 0, bottom: 0 },
  );
}

function mergeSavedRects(rects: AnnotationRect[], width: number, height: number): AnnotationRect[] {
  return mergeSelectionRects(rects.map((rect) => ({
    left: rect.left * width,
    top: rect.top * height,
    right: (rect.left + rect.width) * width,
    bottom: (rect.top + rect.height) * height,
  }))).map((rect) => ({
    left: rect.left / width,
    top: rect.top / height,
    width: (rect.right - rect.left) / width,
    height: (rect.bottom - rect.top) / height,
  }));
}
