import { useCallback, useEffect, useLayoutEffect, useRef, type UIEventHandler } from "react";
import type { MarkdownOutlineNavigationRequest, MarkdownViewMode } from "../types/markdownWorkspace";

type ScrollLock = "editor" | "preview" | "all" | null;

export function useMarkdownScrollSync({ content, navigationRequest, viewMode }: {
  content: string;
  navigationRequest: MarkdownOutlineNavigationRequest | null;
  viewMode: MarkdownViewMode;
}) {
  const editorRef = useRef<HTMLTextAreaElement | null>(null);
  const previewRef = useRef<HTMLDivElement | null>(null);
  const contentRef = useRef(content);
  contentRef.current = content;
  const lockRef = useRef<ScrollLock>(null);
  const releaseFrameRef = useRef<number | null>(null);

  const releaseLock = useCallback((afterTwoFrames = false) => {
    if (releaseFrameRef.current !== null) window.cancelAnimationFrame(releaseFrameRef.current);
    releaseFrameRef.current = window.requestAnimationFrame(() => {
      if (afterTwoFrames) {
        releaseFrameRef.current = window.requestAnimationFrame(() => {
          lockRef.current = null;
          releaseFrameRef.current = null;
        });
      } else {
        lockRef.current = null;
        releaseFrameRef.current = null;
      }
    });
  }, []);

  const synchronize = useCallback((source: HTMLElement, target: HTMLElement, targetName: Exclude<ScrollLock, "all" | null>) => {
    const sourceName = targetName === "editor" ? "preview" : "editor";
    if (lockRef.current === "all" || lockRef.current === sourceName) return;
    const sourceRange = Math.max(0, source.scrollHeight - source.clientHeight);
    const targetRange = Math.max(0, target.scrollHeight - target.clientHeight);
    const progress = sourceRange > 0 ? source.scrollTop / sourceRange : 0;
    lockRef.current = targetName;
    target.scrollTop = progress * targetRange;
    releaseLock();
  }, [releaseLock]);

  const onEditorScroll = useCallback<UIEventHandler<HTMLTextAreaElement>>((event) => {
    const preview = previewRef.current;
    if (preview) synchronize(event.currentTarget, preview, "preview");
  }, [synchronize]);

  const onPreviewScroll = useCallback<UIEventHandler<HTMLDivElement>>((event) => {
    const editor = editorRef.current;
    if (editor) synchronize(event.currentTarget, editor, "editor");
  }, [synchronize]);

  useLayoutEffect(() => {
    if (!navigationRequest) return;
    lockRef.current = "all";
    if (editorRef.current) scrollEditorToOffset(editorRef.current, contentRef.current, navigationRequest.offset);
    if (previewRef.current) scrollPreviewToNode(previewRef.current, navigationRequest.nodeId);
    releaseLock(true);
  }, [navigationRequest, releaseLock, viewMode]);

  useEffect(() => () => {
    if (releaseFrameRef.current !== null) window.cancelAnimationFrame(releaseFrameRef.current);
  }, []);

  return { editorRef, previewRef, onEditorScroll, onPreviewScroll };
}

function scrollEditorToOffset(editor: HTMLTextAreaElement, content: string, offset: number) {
  const safeOffset = Math.max(0, Math.min(offset, content.length));
  const lineIndex = content.slice(0, safeOffset).split(/\r\n|\r|\n/).length - 1;
  const style = window.getComputedStyle(editor);
  const fontSize = Number.parseFloat(style.fontSize) || 15;
  const lineHeight = Number.parseFloat(style.lineHeight) || fontSize * 1.72;
  const paddingTop = Number.parseFloat(style.paddingTop) || 0;
  const target = paddingTop + lineIndex * lineHeight - editor.clientHeight / 2 + lineHeight / 2;
  editor.scrollTop = clampScrollTop(editor, target);
}

function scrollPreviewToNode(preview: HTMLDivElement, nodeId: string) {
  const target = preview.querySelector<HTMLElement>(`#${window.CSS.escape(nodeId)}`);
  if (!target) return;
  const previewBounds = preview.getBoundingClientRect();
  const targetBounds = target.getBoundingClientRect();
  const top = preview.scrollTop + targetBounds.top - previewBounds.top
    - preview.clientHeight / 2 + targetBounds.height / 2;
  preview.scrollTop = clampScrollTop(preview, top);
}

function clampScrollTop(element: HTMLElement, value: number) {
  return Math.max(0, Math.min(value, Math.max(0, element.scrollHeight - element.clientHeight)));
}
