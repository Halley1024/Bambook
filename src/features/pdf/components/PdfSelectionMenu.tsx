import { BookmarkPlus, Copy } from "lucide-react";
import type { AnnotationRect, MarkupType } from "../../annotations";
import { MarkupIcon } from "./MarkupIcon";

export type PdfSelection = {
  x: number;
  y: number;
  page: number;
  text: string;
  rects: AnnotationRect[];
  bookmarkX: number;
  bookmarkY: number;
};

export function PdfSelectionMenu({ selection, onCopy, onApply, onAddBookmark }: {
  selection: PdfSelection;
  onCopy: (selection: PdfSelection) => void;
  onApply: (type: MarkupType, selection: PdfSelection) => void;
  onAddBookmark: (selection: PdfSelection) => void;
}) {
  return <div className="selection-context-menu" style={{ left: selection.x, top: selection.y }}
    onPointerDown={(event) => event.stopPropagation()}>
    <button data-tooltip="复制文字：将所选内容写入剪贴板" onClick={() => onCopy(selection)}><Copy size={17} />复制文字</button>
    <div className="context-menu-separator" />
    <button data-tooltip="添加下划线：标记所选文字" onClick={() => onApply("underline", selection)}><MarkupIcon type="underline" size={17} />添加下划线</button>
    <button data-tooltip="添加波浪线：标记所选文字" onClick={() => onApply("squiggly", selection)}><MarkupIcon type="squiggly" size={17} />添加波浪线</button>
    <button data-tooltip="添加删除线：标记所选文字" onClick={() => onApply("strikeout", selection)}><MarkupIcon type="strikeout" size={17} />添加删除线</button>
    <button data-tooltip="添加高亮：突出显示所选文字" onClick={() => onApply("highlight", selection)}><MarkupIcon type="highlight" size={17} />添加高亮显示</button>
    <div className="context-menu-separator" />
    <button data-tooltip="添加书签：记录当前文字位置" onClick={() => onAddBookmark(selection)}><BookmarkPlus size={17} />添加书签</button>
  </div>;
}
