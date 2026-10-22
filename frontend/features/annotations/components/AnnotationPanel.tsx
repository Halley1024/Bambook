import { useEffect, useMemo, useRef, useState, type CSSProperties, type PointerEvent } from "react";
import { Languages, Merge, Trash2 } from "lucide-react";
import type { Annotation, AnnotationType } from "../types";
import { useSettings, type TranslationProvider } from "../../settings";
import type { TranslationResult } from "../../../platform/translationClient";
import { AssistantSidebar } from "../../../components/ai/AssistantSidebar";

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
  activeTab?: "translation" | "llm";
  onTabChange?: (tab: "translation" | "llm") => void;
  translationText?: string;
  translationResult?: TranslationResult | null;
  translationStatus?: "idle" | "loading" | "success" | "error";
  translationError?: string;
  onTranslate?: () => void;
  llmText?: string;
};

export function AnnotationPanel({ collapsed, onToggleCollapsed, onResizeStart, llmText = "" }: AnnotationPanelProps) {
  return <AssistantSidebar
    contextText={llmText}
    contextLabel="PDF 选中文本"
    emptyContextHint="在 PDF 中选中文字，右键选择“询问大模型”以带入上下文。"
    collapsed={collapsed}
    onToggleCollapsed={onToggleCollapsed}
    onResizeStart={onResizeStart}
  />;
}

export function AnnotationNotesPanel({ annotations, onUpdate, onDelete, onMerge, selectedAnnotationId, onSelect }: {
  annotations: Annotation[];
  onUpdate: (annotation: Annotation) => void;
  onDelete: (id: string) => void;
  onMerge?: (ids: string[]) => void;
  selectedAnnotationId?: string | null;
  onSelect?: (id: string) => void;
}) {
  const cardRefs = useRef(new Map<string, HTMLElement>());
  const [mergeSelection, setMergeSelection] = useState<Set<string>>(new Set());
  const [typeFilter, setTypeFilter] = useState<NoteTypeFilter>(null);
  const orderedAnnotations = useMemo(() => [...annotations].sort(compareAnnotationsByDocumentPosition), [annotations]);
  useEffect(() => {
    if (selectedAnnotationId) cardRefs.current.get(selectedAnnotationId)?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [selectedAnnotationId]);
  useEffect(() => setMergeSelection((current) => new Set([...current].filter((id) => annotations.some((item) => item.id === id)))), [annotations]);

  return <div className="notes-panel">
        <section className="notes-status-section">
          <header><strong>状态</strong><small>{typeFilter ? `正在筛选：${annotationTypeLabel(typeFilter)}` : "点击类型筛选"}</small></header>
          <div className="annotation-stat-grid">
            {noteTypeFilters.map((item) => {
              const count = annotations.filter((annotation) => annotation.type === item.type).length;
              const active = typeFilter === item.type;
              return <button key={item.type} type="button" className={active ? `active ${item.type}` : item.type}
                aria-pressed={active} onClick={() => {
                  setTypeFilter((current) => current === item.type ? null : item.type);
                  setMergeSelection(new Set());
                }} data-tooltip={`${item.label}：显示这一类的所有笔记`}>
                <AnnotationStatusIcon type={item.type} />
                <span className="annotation-stat-copy"><small>{item.label}</small><strong>{count}</strong></span>
              </button>;
            })}
          </div>
        </section>
        <section className="notes-list-section">
          <header className="notes-list-heading"><div><strong>笔记</strong><small>{typeFilter ? `${orderedAnnotations.filter((item) => item.type === typeFilter).length} 条筛选结果` : `共 ${annotations.length} 条`}</small></div>
            <div className="notes-list-actions">
              {onMerge && <button className="annotation-merge-button" disabled={mergeSelection.size < 2}
                onClick={() => { onMerge([...mergeSelection]); setMergeSelection(new Set()); }}
                data-tooltip="合并笔记：将同一页选中的批注合并为一条"><Merge size={15} />合并</button>}
              <button className="annotation-delete-selected-button" disabled={mergeSelection.size === 0}
                onClick={() => {
                  [...mergeSelection].forEach(onDelete);
                  setMergeSelection(new Set());
                }} data-tooltip="删除笔记：删除所有已勾选的笔记"><Trash2 size={15} />删除</button>
            </div>
          </header>
          <div className="annotation-list">
        {annotations.length === 0 ? <p className="empty-text">选中文本后添加画线或笔记。</p>
          : orderedAnnotations.filter((annotation) => !typeFilter || annotation.type === typeFilter).length === 0
            ? <p className="empty-text">当前类型还没有笔记。</p> : (
          orderedAnnotations.filter((annotation) => !typeFilter || annotation.type === typeFilter).map((annotation) => {
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
        </section>
      </div>;
}

type NoteTypeFilter = Exclude<AnnotationType, "note"> | null;
const noteTypeFilters: Array<{ type: Exclude<AnnotationType, "note">; label: string }> = [
  { type: "underline", label: "下划线" },
  { type: "squiggly", label: "波浪线" },
  { type: "strikeout", label: "删除线" },
  { type: "highlight", label: "高亮" },
  { type: "area", label: "区域" },
];

function AnnotationStatusIcon({ type }: { type: Exclude<AnnotationType, "note"> }) {
  if (type === "area") {
    return <span className="annotation-kind-color-icon area annotation-status-type-icon" aria-hidden="true"><i /></span>;
  }
  return <span className={`annotation-kind-color-icon ${type} annotation-status-type-icon`} aria-hidden="true"><b>A</b></span>;
}

const translationProviders: Array<{ value: TranslationProvider; label: string }> = [{ value: "baidu", label: "百度翻译" }];

const translationLanguages = [
  { value: "zh", label: "简体中文" }, { value: "en", label: "English" },
  { value: "ja", label: "日本語" }, { value: "ko", label: "한국어" },
  { value: "fr", label: "Français" }, { value: "de", label: "Deutsch" },
];

function TranslationPanel({ text, result, status, error, onTranslate }: { text: string; result?: TranslationResult | null;
  status: "idle" | "loading" | "success" | "error"; error: string; onTranslate?: () => void }) {
  const { settings, updateSettings } = useSettings();
  const characterCount = Array.from(text).length;
  return <div className="translation-panel">
    <section className="translation-section">
      <header><strong>操作</strong><small>接口与目标语言</small></header>
      <label><span>翻译接口</span><select value={settings.translationProvider}
        onChange={(event) => void updateSettings({ translationProvider: event.target.value as TranslationProvider })}>
        {translationProviders.map((provider) => <option key={provider.value} value={provider.value}>{provider.label}</option>)}
      </select></label>
      <label><span>目标语言</span><select value={settings.translationTargetLanguage}
        onChange={(event) => void updateSettings({ translationTargetLanguage: event.target.value })}>
        {translationLanguages.map((language) => <option key={language.value} value={language.value}>{language.label}</option>)}
      </select></label>
      <button type="button" className="translation-submit-button" disabled={!text.trim() || status === "loading" || !settings.allowThirdPartyTranslation}
        onClick={onTranslate}><Languages size={15} />{status === "loading" ? "正在翻译…" : result ? "重新翻译" : "开始翻译"}</button>
    </section>
    <section className="translation-section translation-result-section">
      <header><strong>译文</strong><small>{status === "loading" ? "正在翻译" : status === "success" ? "翻译完成" : status === "error" ? "翻译失败" : "尚未选择文字"}</small></header>
      {text ? <><div className="translation-source"><span>原文</span><p>{text}</p></div>
        {result ? <div className="translation-output"><span>译文</span><p>{result.translatedText}</p></div>
          : <div className={`translation-placeholder ${status}`}><Languages size={22} /><strong>{status === "loading" ? "正在请求百度翻译…" : status === "error" ? "翻译失败" : "等待翻译"}</strong>
          <p>{status === "error" ? error : "请稍候，译文将在这里显示。"}</p></div>}</>
        : <div className="translation-placeholder"><Languages size={22} /><strong>选择一段文字开始翻译</strong>
          <p>在 PDF 页面中选中文字，右键选择“翻译”。</p></div>}
    </section>
    <section className="translation-section translation-status-section">
      <header><strong>状态</strong><small>当前会话</small></header>
      <dl><div><dt>本次翻译字数</dt><dd>{result?.characterCount ?? characterCount}</dd></div><div><dt>已使用翻译次数</dt><dd>{result?.usageCount ?? 0}</dd></div></dl>
      <div className="translation-quota-label"><span>剩余额度</span><strong>百度接口未提供查询</strong></div>
      <div className="translation-quota-track" role="progressbar" aria-label="剩余翻译额度" aria-valuemin={0} aria-valuemax={100} aria-valuenow={0}><i /></div>
      {!settings.allowThirdPartyTranslation && <p className="translation-privacy-warning">第三方发送权限尚未开启，请在设置中心启用后使用。</p>}
    </section>
  </div>;
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
