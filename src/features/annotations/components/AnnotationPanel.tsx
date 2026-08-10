import { useEffect, useMemo, useRef, useState, type CSSProperties, type PointerEvent } from "react";
import { Merge, PanelRightClose, PanelRightOpen, Trash2 } from "lucide-react";
import type { Annotation } from "../types";

type AnnotationPanelProps = {
  annotations: Annotation[];
  onUpdate: (annotation: Annotation) => void;
  onDelete: (id: string) => void;
  onMerge?: (ids: string[]) => void;
  selectedAnnotationId?: string | null;
  onSelect?: (id: string) => void;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onResizeStart?: (event: PointerEvent) => void;
};

export function AnnotationPanel({ annotations, onUpdate, onDelete, onMerge, selectedAnnotationId, onSelect, collapsed, onToggleCollapsed, onResizeStart }: AnnotationPanelProps) {
  const cardRefs = useRef(new Map<string, HTMLElement>());
  const [mergeSelection, setMergeSelection] = useState<Set<string>>(new Set());
  const orderedAnnotations = useMemo(
    () => [...annotations].sort(compareAnnotationsByDocumentPosition),
    [annotations],
  );
  useEffect(() => {
    if (selectedAnnotationId) cardRefs.current.get(selectedAnnotationId)?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [selectedAnnotationId]);
  useEffect(() => setMergeSelection((current) => new Set([...current].filter((id) => annotations.some((item) => item.id === id)))), [annotations]);
  return (
    <aside className={`side-panel ${collapsed ? "collapsed" : ""}`} aria-label="文档检查器">
      {collapsed ? (
        <button className="annotation-expand-button" onClick={onToggleCollapsed} data-tooltip="展开文档检查器：显示 PDF 笔记">
          <PanelRightOpen size={19} />
          <span className="annotation-collapsed-count">{annotations.length}</span>
          <span className="annotation-collapsed-label">文档检查器</span>
        </button>
      ) : (
        <>
      <div className="panel-heading">
        <div className="panel-heading-title">
          <button className="annotation-collapse-button" onClick={onToggleCollapsed} data-tooltip="收起文档检查器">
            <PanelRightClose size={18} />
          </button>
          <h2>文档检查器</h2>
        </div>
        <div className="annotation-heading-actions">
          {onMerge && <button className="annotation-merge-button" disabled={mergeSelection.size < 2}
            onClick={() => { onMerge([...mergeSelection]); setMergeSelection(new Set()); }}
            data-tooltip="合并笔记：将同一页选中的批注合并为一条"><Merge size={15} />合并</button>}
          <span>{annotations.length}</span>
        </div>
      </div>
      <div className="annotation-list">
        {annotations.length === 0 ? (
          <p className="empty-text">选中文本后添加画线或笔记。</p>
        ) : (
          orderedAnnotations.map((annotation) => {
            const selected = selectedAnnotationId === annotation.id;
            return (
            <article ref={(element) => { if (element) cardRefs.current.set(annotation.id, element); else cardRefs.current.delete(annotation.id); }}
              className={`annotation-card ${selected ? "selected" : ""}`} key={annotation.id}
              onClick={() => { if (!selected) onSelect?.(annotation.id); }}>
              <div className="annotation-meta">
                {onMerge && <input type="checkbox" className="annotation-merge-checkbox" checked={mergeSelection.has(annotation.id)}
                  aria-label={`选择合并第 ${annotation.page ?? 0} 页批注`} onClick={(event) => event.stopPropagation()}
                  onChange={(event) => setMergeSelection((current) => {
                    const next = new Set(current); if (event.target.checked) next.add(annotation.id); else next.delete(annotation.id); return next;
                  })} />}
                <AnnotationColorIcon annotation={annotation} />
                <span>{annotation.page ? `第 ${annotation.page} 页` : "Markdown"}</span>
                <span className="annotation-type-label">{annotationTypeLabel(annotation.type)}</span>
                <button className="small-icon-button" onClick={(event) => {
                  event.stopPropagation();
                  onDelete(annotation.id);
                }} data-tooltip="删除：移除这条批注和笔记" aria-label="删除批注">
                  <Trash2 size={15} />
                </button>
              </div>
              <EditableAnnotationExcerpt annotation={annotation} expanded={selected} onUpdate={onUpdate} onSelect={onSelect} />
              <DebouncedNoteEditor annotation={annotation} expanded={selected} onUpdate={onUpdate} onSelect={onSelect} />
            </article>
          );})
        )}
      </div>
        </>
      )}
      {!collapsed && onResizeStart && (
        <div className="panel-resize-handle left-edge" role="separator" aria-orientation="vertical"
          aria-label="调整文档检查器宽度" onPointerDown={onResizeStart} />
      )}
    </aside>
  );
}

function AnnotationColorIcon({ annotation }: { annotation: Annotation }) {
  const style = {
    "--annotation-icon-color": annotation.color,
    "--annotation-fill-color": annotation.areaStyle?.backgroundColor ?? annotation.color,
  } as CSSProperties;
  const label = `${annotationTypeLabel(annotation.type)}，颜色 ${annotation.color}`;
  if (annotation.type === "area") {
    return <span className="annotation-kind-color-icon area" style={style} aria-label={label} data-tooltip={label}>
      <i />
    </span>;
  }
  return <span className={`annotation-kind-color-icon ${annotation.type}`} style={style} aria-label={label} data-tooltip={label}>
    {annotation.type === "note" ? <i /> : <b>A</b>}
  </span>;
}

const COLLAPSED_EXCERPT_LENGTH = 200;
const MINIMUM_EXCERPT_EDITOR_HEIGHT = 96;

function EditableAnnotationExcerpt({ annotation, expanded, onUpdate, onSelect }: {
  annotation: Annotation;
  expanded: boolean;
  onUpdate: (annotation: Annotation) => void;
  onSelect?: (id: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(annotation.selectedText);
  const [editorHeight, setEditorHeight] = useState<number | null>(null);
  useEffect(() => setDraft(annotation.selectedText), [annotation.selectedText]);
  useEffect(() => { if (!expanded) setEditing(false); }, [expanded]);

  function commit() {
    setEditing(false);
    if (draft === annotation.selectedText) return;
    onUpdate({ ...annotation, selectedText: draft, updatedAt: new Date().toISOString() });
  }

  if (editing) {
    return <textarea className="annotation-excerpt-editor" autoFocus value={draft} aria-label="编辑选中文字"
      style={editorHeight ? { height: editorHeight } : undefined}
      onClick={(event) => event.stopPropagation()} onChange={(event) => setDraft(event.target.value)} onBlur={commit}
      onKeyDown={(event) => {
        if ((event.ctrlKey || event.metaKey) && event.key === "Enter") event.currentTarget.blur();
      }} />;
  }

  const excerpt = expanded || annotation.selectedText.length <= COLLAPSED_EXCERPT_LENGTH
    ? annotation.selectedText
    : `${annotation.selectedText.slice(0, COLLAPSED_EXCERPT_LENGTH).trimEnd()}…`;
  return <blockquote className={expanded ? "expanded" : "collapsed"}
    onDoubleClick={(event) => {
      event.stopPropagation();
      if (!expanded) onSelect?.(annotation.id);
      setEditorHeight(Math.max(MINIMUM_EXCERPT_EDITOR_HEIGHT, event.currentTarget.getBoundingClientRect().height));
      setEditing(true);
    }}>{excerpt}</blockquote>;
}

function DebouncedNoteEditor({ annotation, expanded, onUpdate, onSelect }: {
  annotation: Annotation;
  expanded: boolean;
  onUpdate: (annotation: Annotation) => void;
  onSelect?: (id: string) => void;
}) {
  const [draft, setDraft] = useState(annotation.note);
  const draftRef = useRef(draft);
  const timerRef = useRef<number | null>(null);
  const flushRef = useRef<() => void>(() => undefined);
  draftRef.current = draft;
  useEffect(() => { setDraft(annotation.note); draftRef.current = annotation.note; }, [annotation.note]);
  const flush = () => {
    if (timerRef.current !== null) { window.clearTimeout(timerRef.current); timerRef.current = null; }
    if (draftRef.current === annotation.note) return;
    onUpdate({ ...annotation, note: draftRef.current, hasNote: Boolean(draftRef.current.trim()), updatedAt: new Date().toISOString() });
  };
  flushRef.current = flush;
  useEffect(() => {
    const flushOnSave = () => flushRef.current();
    window.addEventListener("bambook:flush-note-editors", flushOnSave);
    return () => { window.removeEventListener("bambook:flush-note-editors", flushOnSave); if (timerRef.current !== null) window.clearTimeout(timerRef.current); };
  }, []);
  return <textarea className="annotation-note-editor" rows={expanded ? 4 : 1} value={draft} placeholder="添加批注"
    onFocus={() => { if (!expanded) onSelect?.(annotation.id); }} onBlur={flush}
    onChange={(event) => {
      const value = event.target.value; setDraft(value); draftRef.current = value;
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(flush, 350);
    }} />;
}

function compareAnnotationsByDocumentPosition(left: Annotation, right: Annotation) {
  const pageDifference = (left.page ?? Number.MAX_SAFE_INTEGER) - (right.page ?? Number.MAX_SAFE_INTEGER);
  if (pageDifference !== 0) return pageDifference;

  const leftPosition = firstRectPosition(left);
  const rightPosition = firstRectPosition(right);
  const topDifference = leftPosition.top - rightPosition.top;
  if (topDifference !== 0) return topDifference;

  const horizontalDifference = leftPosition.left - rightPosition.left;
  if (horizontalDifference !== 0) return horizontalDifference;

  return left.createdAt.localeCompare(right.createdAt) || left.id.localeCompare(right.id);
}

function firstRectPosition(annotation: Annotation) {
  if (!annotation.rects?.length) {
    return { top: Number.MAX_SAFE_INTEGER, left: Number.MAX_SAFE_INTEGER };
  }

  return annotation.rects.reduce(
    (first, rect) => rect.top < first.top || (rect.top === first.top && rect.left < first.left)
      ? { top: rect.top, left: rect.left }
      : first,
    { top: Number.MAX_SAFE_INTEGER, left: Number.MAX_SAFE_INTEGER },
  );
}

function annotationTypeLabel(type: Annotation["type"]) {
  if (type === "area") return "选择区域";
  if (type === "underline") return "下划线";
  if (type === "squiggly") return "波浪线";
  if (type === "strikeout") return "删除线";
  if (type === "highlight") return "高亮";
  return "笔记";
}
