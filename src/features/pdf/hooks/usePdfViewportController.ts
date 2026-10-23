import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { clamp } from "../../../shared/utils/clamp";
import type { PdfNavigationTarget } from "../types/pdfNavigation";
import { PDF_MAX_SCALE, PDF_MIN_SCALE } from "../pdfZoom";

export function usePdfViewportController(
  pageCount: number,
  scale: number,
  navigationTarget: PdfNavigationTarget | null,
  onCurrentPageChange: (page: number, pageY: number, pageX: number) => void,
  onScaleChange: (scale: number) => void,
) {
  const [fitWidth, setFitWidth] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);
  const [renderCenterPage, setRenderCenterPage] = useState(1);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const previousScaleRef = useRef(scale);
  const currentScaleRef = useRef(scale);
  const previousFitWidthRef = useRef(0);
  const reportedPageRef = useRef(1);
  const navigationLockRef = useRef<number | null>(null);
  const zoomLayoutLockRef = useRef(false);
  const viewportAnchorRef = useRef<{ pageNumber: string; x: number; y: number; offsetX?: number; offsetY?: number } | null>(null);
  const pageChangeRef = useRef(onCurrentPageChange);
  const scaleChangeRef = useRef(onScaleChange);
  pageChangeRef.current = onCurrentPageChange;
  scaleChangeRef.current = onScaleChange;
  currentScaleRef.current = scale;

  useEffect(() => {
    const scroller = scrollRef.current;
    if (!scroller || pageCount === 0) return;

    const updateFitWidth = () => {
      const bounds = scroller.getBoundingClientRect();
      setFitWidth(Math.max(240, Math.floor(bounds.width - 64)));
      setViewportHeight(Math.max(240, Math.floor(bounds.height)));
    };

    updateFitWidth();
    const visibleAreas = new Map<number, number>();
    let lastReadingPosition: { page: number; x: number; y: number } | null = null;
    const reportPosition = (pageNumber: number) => {
      const position = captureReadingPosition(scroller, pageNumber);
      if (!position) return;
      if (lastReadingPosition?.page === pageNumber
        && Math.abs(lastReadingPosition.x - position.x) < 0.005
        && Math.abs(lastReadingPosition.y - position.y) < 0.005) return;
      lastReadingPosition = { page: pageNumber, x: position.x, y: position.y };
      pageChangeRef.current(pageNumber, position.y, position.x);
    };
    const observer = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        const pageNumber = Number((entry.target as HTMLElement).dataset.pageNumber ?? 1);
        visibleAreas.set(pageNumber, entry.isIntersecting
          ? entry.intersectionRect.width * entry.intersectionRect.height : 0);
      });
      if (navigationLockRef.current !== null || zoomLayoutLockRef.current) return;
      let largestPage = reportedPageRef.current;
      let largestArea = 0;
      visibleAreas.forEach((area, page) => {
        if (area > largestArea) { largestArea = area; largestPage = page; }
      });
      const currentArea = visibleAreas.get(reportedPageRef.current) ?? 0;
      if (largestPage !== reportedPageRef.current && (currentArea === 0 || largestArea > currentArea * 1.04)) {
        reportedPageRef.current = largestPage;
        setRenderCenterPage(largestPage);
        reportPosition(largestPage);
      }
      viewportAnchorRef.current = captureViewportAnchor(scroller, reportedPageRef.current);
    }, {
      root: scroller,
      threshold: Array.from({ length: 21 }, (_, index) => index / 20),
    });
    scroller.querySelectorAll<HTMLElement>(".pdf-page").forEach((page) => observer.observe(page));
    const resizeObserver = new ResizeObserver(() => {
      updateFitWidth();
    });
    let scrollFrame = 0;
    const reportReadingPosition = () => {
      window.cancelAnimationFrame(scrollFrame);
      scrollFrame = window.requestAnimationFrame(() => {
        reportPosition(reportedPageRef.current);
      });
    };
    scroller.addEventListener("scroll", reportReadingPosition, { passive: true });
    const handleWheelZoom = (event: WheelEvent) => {
      if (!event.ctrlKey) return;
      event.preventDefault();
      const scrollerBounds = scroller.getBoundingClientRect();
      const pointerInsideViewer = pointInsideRect(event.clientX, event.clientY, scrollerBounds);
      const anchorClientX = pointerInsideViewer ? event.clientX : scrollerBounds.left + scrollerBounds.width / 2;
      const anchorClientY = pointerInsideViewer ? event.clientY : scrollerBounds.top + scrollerBounds.height / 2;
      const anchor = captureAnchorAtPoint(scroller, anchorClientX, anchorClientY, reportedPageRef.current);
      if (!anchor) return;
      viewportAnchorRef.current = anchor;
      const factor = Math.exp(-event.deltaY * 0.002);
      const currentScale = currentScaleRef.current;
      const nextScale = Math.round(clamp(currentScale * factor, PDF_MIN_SCALE, PDF_MAX_SCALE) * 100) / 100;
      if (nextScale !== currentScale) {
        currentScaleRef.current = nextScale;
        scaleChangeRef.current(nextScale);
      }
    };
    window.addEventListener("wheel", handleWheelZoom, { passive: false, capture: true });
    resizeObserver.observe(scroller);
    return () => {
      window.cancelAnimationFrame(scrollFrame);
      scroller.removeEventListener("scroll", reportReadingPosition);
      window.removeEventListener("wheel", handleWheelZoom, { capture: true });
      observer.disconnect();
      resizeObserver.disconnect();
    };
  }, [pageCount]);

  useEffect(() => {
    const scroller = scrollRef.current;
    if (!scroller || !navigationTarget) return;
    setRenderCenterPage(navigationTarget.page);
    const page = scroller.querySelector<HTMLElement>(`[data-page-number="${navigationTarget.page}"]`);
    if (!page) return;
    reportedPageRef.current = navigationTarget.page;
    navigationLockRef.current = navigationTarget.page;
    const scrollerBounds = scroller.getBoundingClientRect();
    const pageBounds = page.getBoundingClientRect();
    const paper = page.querySelector<HTMLElement>(".pdf-paper");
    const paperBounds = paper?.getBoundingClientRect() ?? pageBounds;
    const targetTop = navigationTarget.pageY === undefined
      ? scroller.scrollTop + pageBounds.top - scrollerBounds.top - 28
      : scroller.scrollTop + paperBounds.top - scrollerBounds.top
        + clamp(navigationTarget.pageY, 0, 1) * paperBounds.height
        - scrollerBounds.height / 2;
    const targetLeft = navigationTarget.pageX === undefined
      ? scroller.scrollLeft
      : scroller.scrollLeft + paperBounds.left - scrollerBounds.left
        + clamp(navigationTarget.pageX, 0, 1) * paperBounds.width
        - scrollerBounds.width / 2;
    scroller.scrollTo({ left: Math.max(0, targetLeft), top: Math.max(0, targetTop), behavior: "smooth" });

    let released = false;
    const releaseNavigationLock = () => {
      if (released) return;
      released = true;
      navigationLockRef.current = null;
      scroller.dispatchEvent(new Event("scroll"));
    };
    scroller.addEventListener("scrollend", releaseNavigationLock, { once: true });
    const fallback = window.setTimeout(releaseNavigationLock, 1200);
    return () => {
      released = true;
      window.clearTimeout(fallback);
      scroller.removeEventListener("scrollend", releaseNavigationLock);
      navigationLockRef.current = null;
    };
  }, [navigationTarget]);

  useLayoutEffect(() => {
    const previousScale = previousScaleRef.current;
    const previousFitWidth = previousFitWidthRef.current;
    previousScaleRef.current = scale;
    previousFitWidthRef.current = fitWidth;
    const scroller = scrollRef.current;
    if (!scroller || previousFitWidth === 0 || (previousScale === scale && previousFitWidth === fitWidth)) return;
    const anchor = viewportAnchorRef.current;
    if (!anchor) return;
    const targetPage = scroller.querySelector<HTMLElement>(`[data-page-number="${anchor.pageNumber}"]`);
    const targetPaper = targetPage?.querySelector<HTMLElement>(".pdf-paper");
    if (!targetPaper) return;

    zoomLayoutLockRef.current = true;
    const scrollerBounds = scroller.getBoundingClientRect();
    const targetBounds = targetPaper.getBoundingClientRect();
    const nextLeft = scroller.scrollLeft + targetBounds.left + anchor.x * targetBounds.width
      - scrollerBounds.left - (anchor.offsetX ?? 0);
    const nextTop = scroller.scrollTop + targetBounds.top + anchor.y * targetBounds.height
      - scrollerBounds.top - (anchor.offsetY ?? 0);
    scroller.scrollTo({ left: nextLeft, top: nextTop, behavior: "auto" });

    let frame = window.requestAnimationFrame(() => {
      frame = window.requestAnimationFrame(() => {
        zoomLayoutLockRef.current = false;
        viewportAnchorRef.current = captureViewportAnchor(scroller, reportedPageRef.current);
        scroller.dispatchEvent(new Event("scroll"));
      });
    });
    return () => {
      window.cancelAnimationFrame(frame);
      zoomLayoutLockRef.current = false;
    };
  }, [fitWidth, scale]);

  return { fitWidth, viewportHeight, renderCenterPage, scrollRef };
}

function pointInsideRect(x: number, y: number, rect: DOMRect) {
  return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
}

function captureAnchorAtPoint(scroller: HTMLElement, clientX: number, clientY: number, preferredPage: number) {
  const scrollerBounds = scroller.getBoundingClientRect();
  const papers = [...scroller.querySelectorAll<HTMLElement>(".pdf-paper")];
  if (!papers.length) return null;

  const preferredPaper = scroller.querySelector<HTMLElement>(`[data-page-number="${preferredPage}"] .pdf-paper`);
  const paper = papers.find((candidate) => pointInsideRect(clientX, clientY, candidate.getBoundingClientRect()))
    ?? nearestPaperToPoint(papers, clientX, clientY)
    ?? preferredPaper;
  const page = paper?.closest<HTMLElement>(".pdf-page");
  if (!paper || !page) return null;
  const paperBounds = paper.getBoundingClientRect();

  return {
    pageNumber: page.dataset.pageNumber ?? String(preferredPage),
    x: clamp((clientX - paperBounds.left) / paperBounds.width, 0, 1),
    y: clamp((clientY - paperBounds.top) / paperBounds.height, 0, 1),
    offsetX: clamp(clientX - scrollerBounds.left, 0, scrollerBounds.width),
    offsetY: clamp(clientY - scrollerBounds.top, 0, scrollerBounds.height),
  };
}

function nearestPaperToPoint(papers: HTMLElement[], x: number, y: number) {
  let nearest: HTMLElement | null = null;
  let nearestDistance = Number.POSITIVE_INFINITY;
  papers.forEach((paper) => {
    const bounds = paper.getBoundingClientRect();
    const dx = x < bounds.left ? bounds.left - x : x > bounds.right ? x - bounds.right : 0;
    const dy = y < bounds.top ? bounds.top - y : y > bounds.bottom ? y - bounds.bottom : 0;
    const distance = dx * dx + dy * dy;
    if (distance < nearestDistance) {
      nearest = paper;
      nearestDistance = distance;
    }
  });
  return nearest;
}

function captureReadingPosition(scroller: HTMLElement, pageNumber: number) {
  const viewport = scroller.getBoundingClientRect();
  const page = scroller.querySelector<HTMLElement>(`[data-page-number="${pageNumber}"]`);
  const paper = page?.querySelector<HTMLElement>(".pdf-paper");
  if (!paper) return null;
  const bounds = paper.getBoundingClientRect();
  return {
    x: clamp((viewport.left + viewport.width / 2 - bounds.left) / bounds.width, 0, 1),
    y: clamp((viewport.top + viewport.height / 2 - bounds.top) / bounds.height, 0, 1),
  };
}

function captureViewportAnchor(scroller: HTMLElement, pageNumber: number) {
  const viewport = scroller.getBoundingClientRect();
  const page = scroller.querySelector<HTMLElement>(`[data-page-number="${pageNumber}"]`);
  const paper = page?.querySelector<HTMLElement>(".pdf-paper");
  if (!page || !paper) return null;
  const bounds = paper.getBoundingClientRect();
  return {
    pageNumber: page.dataset.pageNumber ?? "1",
    x: clamp((viewport.left - bounds.left) / bounds.width, 0, 1),
    y: clamp((viewport.top - bounds.top) / bounds.height, 0, 1),
    offsetX: 0,
    offsetY: 0,
  };
}
