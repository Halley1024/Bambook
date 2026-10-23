import type { AnnotationRect } from "../../annotations";

export type PixelSelectionRect = { left: number; top: number; right: number; bottom: number };
type TextRunCharacter = { text: string; rect: AnnotationRect };
export type TextAnnotationResizeEdge = "start" | "end";
export type ResizedTextAnnotation = { text: string; rects: AnnotationRect[] };
const textRunGeometry = new WeakMap<HTMLElement, TextRunCharacter[]>();

export function registerTextRunGeometry(element: HTMLElement, characters: TextRunCharacter[]) {
  textRunGeometry.set(element, characters);
}

export function resizeTextAnnotationAt(
  paper: HTMLElement,
  annotationRects: AnnotationRect[],
  edge: TextAnnotationResizeEdge,
  clientX: number,
  clientY: number,
): ResizedTextAnnotation | null {
  const characters = Array.from(paper.querySelectorAll<HTMLElement>(".mupdf-text-run"))
    .flatMap((run) => textRunGeometry.get(run) ?? []);
  if (!characters.length || !annotationRects.length) return null;
  const selectedIndexes = characters.flatMap((character, index) => (
    annotationRects.some((rect) => rectsOverlap(character.rect, rect)) ? [index] : []
  ));
  if (!selectedIndexes.length) return null;
  const bounds = paper.getBoundingClientRect();
  const pointer = {
    x: Math.max(0, Math.min(1, (clientX - bounds.left) / bounds.width)),
    y: Math.max(0, Math.min(1, (clientY - bounds.top) / bounds.height)),
  };
  const targetIndex = nearestCharacterIndex(characters, pointer.x, pointer.y);
  const originalStart = Math.min(...selectedIndexes);
  const originalEnd = Math.max(...selectedIndexes);
  const start = edge === "start" ? Math.min(targetIndex, originalEnd) : originalStart;
  const end = edge === "end" ? Math.max(targetIndex, originalStart) : originalEnd;
  const selected = characters.slice(start, end + 1);
  return {
    text: selectedText(selected),
    rects: mergeSelectionRects(selected.map(({ rect }) => ({
      left: rect.left * bounds.width,
      top: rect.top * bounds.height,
      right: (rect.left + rect.width) * bounds.width,
      bottom: (rect.top + rect.height) * bounds.height,
    }))).map((rect) => ({
      left: rect.left / bounds.width,
      top: rect.top / bounds.height,
      width: (rect.right - rect.left) / bounds.width,
      height: (rect.bottom - rect.top) / bounds.height,
    })),
  };
}

function rectsOverlap(character: AnnotationRect, selection: AnnotationRect) {
  const overlapWidth = Math.min(character.left + character.width, selection.left + selection.width)
    - Math.max(character.left, selection.left);
  const overlapHeight = Math.min(character.top + character.height, selection.top + selection.height)
    - Math.max(character.top, selection.top);
  return overlapWidth > Math.min(character.width, selection.width) * 0.25
    && overlapHeight > Math.min(character.height, selection.height) * 0.45;
}

function nearestCharacterIndex(characters: TextRunCharacter[], x: number, y: number) {
  let nearest = 0;
  let nearestDistance = Number.POSITIVE_INFINITY;
  characters.forEach(({ rect }, index) => {
    const dx = x < rect.left ? rect.left - x : x > rect.left + rect.width ? x - rect.left - rect.width : 0;
    const dy = y < rect.top ? rect.top - y : y > rect.top + rect.height ? y - rect.top - rect.height : 0;
    const distance = dy * 4 + dx;
    if (distance < nearestDistance) { nearest = index; nearestDistance = distance; }
  });
  return nearest;
}

function selectedText(characters: TextRunCharacter[]) {
  let result = "";
  let previous: TextRunCharacter | undefined;
  characters.forEach((character) => {
    if (previous) {
      const overlap = Math.min(previous.rect.top + previous.rect.height, character.rect.top + character.rect.height)
        - Math.max(previous.rect.top, character.rect.top);
      if (overlap < Math.min(previous.rect.height, character.rect.height) * 0.35) result += "\n";
    }
    result += character.text;
    previous = character;
  });
  return result.trim();
}

export function readSelectionRects(selection: Selection, paper: HTMLElement): AnnotationRect[] {
  if (selection.isCollapsed || selection.rangeCount === 0) return [];
  const range = selection.getRangeAt(0);
  try {
    if (!range.intersectsNode(paper)) return [];
  } catch {
    return [];
  }
  const paperBounds = paper.getBoundingClientRect();
  const structuredRects = readStructuredSelectionRects(range, paper, paperBounds);
  if (structuredRects.length) return structuredRects;
  const pixelRects = Array.from(range.getClientRects())
    .filter((rect) => rect.width > 0 && rect.height > 0
      && rect.right > paperBounds.left && rect.left < paperBounds.right
      && rect.bottom > paperBounds.top && rect.top < paperBounds.bottom)
    .map((rect) => ({
      left: Math.max(0, rect.left - paperBounds.left),
      top: Math.max(0, rect.top - paperBounds.top),
      right: Math.min(paperBounds.width, rect.right - paperBounds.left),
      bottom: Math.min(paperBounds.height, rect.bottom - paperBounds.top),
    }));
  return mergeSelectionRects(pixelRects).map((rect) => ({
    left: rect.left / paperBounds.width,
    top: rect.top / paperBounds.height,
    width: (rect.right - rect.left) / paperBounds.width,
    height: (rect.bottom - rect.top) / paperBounds.height,
  }));
}

function readStructuredSelectionRects(range: Range, paper: HTMLElement, paperBounds: DOMRect): AnnotationRect[] {
  const selected: AnnotationRect[] = [];
  paper.querySelectorAll<HTMLElement>(".mupdf-text-run").forEach((run) => {
    const characters = textRunGeometry.get(run);
    if (!characters?.length) return;
    try {
      if (!range.intersectsNode(run)) return;
    } catch {
      return;
    }
    const textLength = run.textContent?.length ?? 0;
    const start = run.contains(range.startContainer) ? textOffset(run, range.startContainer, range.startOffset) : 0;
    const end = run.contains(range.endContainer) ? textOffset(run, range.endContainer, range.endOffset) : textLength;
    let offset = 0;
    characters.forEach((character) => {
      const nextOffset = offset + character.text.length;
      if (nextOffset > start && offset < end) selected.push(character.rect);
      offset = nextOffset;
    });
  });
  if (!selected.length) return [];
  return mergeSelectionRects(selected.map((rect) => ({
    left: rect.left * paperBounds.width,
    top: rect.top * paperBounds.height,
    right: (rect.left + rect.width) * paperBounds.width,
    bottom: (rect.top + rect.height) * paperBounds.height,
  }))).map((rect) => ({
    left: rect.left / paperBounds.width,
    top: rect.top / paperBounds.height,
    width: (rect.right - rect.left) / paperBounds.width,
    height: (rect.bottom - rect.top) / paperBounds.height,
  }));
}

function textOffset(host: HTMLElement, node: Node, offset: number) {
  try {
    const prefix = window.document.createRange();
    prefix.selectNodeContents(host);
    prefix.setEnd(node, offset);
    return prefix.toString().length;
  } catch {
    return 0;
  }
}

export function mergeSelectionRects(rects: PixelSelectionRect[]): PixelSelectionRect[] {
  const unique = rects.filter((rect, index, items) => !items.slice(0, index).some((other) =>
    Math.abs(rect.left - other.left) < 1 && Math.abs(rect.top - other.top) < 1
      && Math.abs(rect.right - other.right) < 1 && Math.abs(rect.bottom - other.bottom) < 1));
  unique.sort((a, b) => a.top - b.top || a.left - b.left);

  const merged: PixelSelectionRect[] = [];
  for (const rect of unique) {
    if (rect.right <= rect.left || rect.bottom <= rect.top) continue;
    const previous = merged[merged.length - 1];
    if (!previous) { merged.push({ ...rect }); continue; }
    const overlap = Math.min(previous.bottom, rect.bottom) - Math.max(previous.top, rect.top);
    const minHeight = Math.min(previous.bottom - previous.top, rect.bottom - rect.top);
    const sameLine = overlap > minHeight * 0.55;
    const gap = rect.left - previous.right;
    const joinDistance = Math.max(3, minHeight * 0.8);
    if (sameLine && gap <= joinDistance) {
      previous.left = Math.min(previous.left, rect.left);
      previous.top = Math.min(previous.top, rect.top);
      previous.right = Math.max(previous.right, rect.right);
      previous.bottom = Math.max(previous.bottom, rect.bottom);
    } else {
      merged.push({ ...rect });
    }
  }
  return merged;
}
