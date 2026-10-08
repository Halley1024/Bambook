import { BookmarkPlus, Copy, Trash2 } from "lucide-react";
import type { Annotation, MarkupType } from "../../annotations";
import { MarkupIcon } from "./MarkupIcon";
import { ANNOTATION_CONTEXT_COLORS, ContextColorPicker } from "./ContextColorPicker";

const MARKUP_TYPES: MarkupType[] = ["highlight", "underline", "squiggly", "strikeout"];

export function PdfAnnotationMenu({ x, y, annotation, onCopy, onColor, onConvert, onDelete, onAddBookmark }: {
  x: number;
  y: number;
  annotation: Annotation;
  onCopy: () => void;
  onColor: (color: string) => void;
  onConvert: (type: MarkupType) => void;
  onDelete: () => void;
  onAddBookmark: () => void;
}) {
  return <div className="selection-context-menu annotation-context-menu" style={{ left: x, top: y }}
    onPointerDown={(event) => event.stopPropagation()}>
    <button data-tooltip="复制文字：将批注对应文字写入剪贴板" onClick={onCopy}><Copy size={17} />复制文字</button>
    <div className="context-menu-separator" />
    <div className="context-menu-label">调色盘</div>
    <ContextColorPicker value={annotation.color} onChange={onColor} scopeLabel="批注"
      colors={ANNOTATION_CONTEXT_COLORS} showLabels={false} dense />
    <div className="context-menu-separator" />
    {MARKUP_TYPES.filter((type) => type !== annotation.type).map((type) => (
      <button key={type} data-tooltip={`转换样式：改为${markupLabel(type)}`} onClick={() => onConvert(type)}>
        <MarkupIcon type={type} size={17} />转为{markupLabel(type)}
      </button>
    ))}
    <button className="danger" data-tooltip={`删除：移除${annotationTypeLabel(annotation.type)}及关联笔记`}
      onClick={onDelete}><Trash2 size={17} />删除{annotationTypeLabel(annotation.type)}</button>
    <div className="context-menu-separator" />
    <button data-tooltip="添加书签：记录当前页面位置" onClick={onAddBookmark}><BookmarkPlus size={17} />添加书签</button>
  </div>;
}

function annotationTypeLabel(type: Annotation["type"]) {
  if (type === "note") return "笔记";
  if (type === "area") return "选择区域";
  return markupLabel(type);
}

function markupLabel(type: MarkupType) {
  if (type === "highlight") return "高亮";
  if (type === "underline") return "下划线";
  if (type === "squiggly") return "波浪线";
  return "删除线";
}
