import { useEffect, useMemo, useRef, useState } from "react";
import type { PdfDocument } from "../../documents";
import type { PdfOutlineNode } from "../types/pdfStructure";

export type OutlineItem = {
  id: string;
  title: string;
  page: number | null;
  children: OutlineItem[];
};

export function usePdfOutline(document: PdfDocument | null, currentPage: number) {
  const outline = useMemo(() => mapOutline(document?.outline ?? []), [document?.outline]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const itemRefs = useRef(new Map<string, HTMLButtonElement>());

  useEffect(() => {
    setExpanded(new Set(collectExpandableIds(outline)));
  }, [outline]);

  const activeId = useMemo(() => findActiveOutlineId(outline, currentPage), [currentPage, outline]);

  useEffect(() => {
    if (!activeId) return;
    const ancestors = findAncestorIds(outline, activeId);
    if (ancestors.length) setExpanded((current) => new Set([...current, ...ancestors]));
    window.requestAnimationFrame(() => itemRefs.current.get(activeId)?.scrollIntoView({ block: "nearest" }));
  }, [activeId, outline]);

  function toggleExpanded(id: string) {
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return { outline, loading: false, expanded, activeId, itemRefs, toggleExpanded };
}

function mapOutline(items: PdfOutlineNode[], prefix = "root"): OutlineItem[] {
  return items.map((item, index) => {
    const id = `${prefix}-${index}`;
    return { id, title: item.title, page: item.page === undefined ? null : item.page + 1, children: mapOutline(item.children, id) };
  });
}

function collectExpandableIds(items: OutlineItem[]): string[] {
  return items.flatMap((item) => (item.children.length ? [item.id, ...collectExpandableIds(item.children)] : []));
}

function flattenOutline(items: OutlineItem[]): OutlineItem[] {
  return items.flatMap((item) => [item, ...flattenOutline(item.children)]);
}

function findActiveOutlineId(items: OutlineItem[], currentPage: number): string | null {
  let active: OutlineItem | null = null;
  for (const item of flattenOutline(items)) {
    if (item.page !== null && item.page <= currentPage && (!active || item.page >= (active.page ?? 0))) active = item;
  }
  return active?.id ?? null;
}

function findAncestorIds(items: OutlineItem[], targetId: string, ancestors: string[] = []): string[] {
  for (const item of items) {
    if (item.id === targetId) return ancestors;
    const found = findAncestorIds(item.children, targetId, [...ancestors, item.id]);
    if (found.length) return found;
  }
  return [];
}
