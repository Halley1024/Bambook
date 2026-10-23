import { useState } from "react";
import { Bookmark, Move, Pencil, Trash2 } from "lucide-react";
import { BookmarkColorPicker } from "./BookmarkColorPicker";
import type { PdfBookmark } from "../types/pdfBookmark";

export function PdfExistingBookmarkMenu({ x, y, bookmark, onUpdate, onMove, onDelete, onClose }: {
  x: number; y: number; bookmark: PdfBookmark;
  onUpdate: (changes: Partial<Pick<PdfBookmark, "title" | "color">>) => void;
  onMove: () => void; onDelete: () => void; onClose: () => void;
}) {
  const [renaming, setRenaming] = useState(false);
  const [title, setTitle] = useState(bookmark.title);
  const commitTitle = () => {
    if (!title.trim()) return;
    onUpdate({ title: title.trim() });
    onClose();
  };
  return <div className="selection-context-menu bookmark-context-menu" style={{ left: x, top: y }}
    onPointerDown={(event) => event.stopPropagation()}>
    <div className="bookmark-context-color">
      <span>书签颜色</span>
      <BookmarkColorPicker value={bookmark.color} onChange={(color) => onUpdate({ color })} />
    </div>
    {renaming ? <label className="bookmark-name-field"><span>书签名称</span>
      <input autoFocus value={title} maxLength={80} onChange={(event) => setTitle(event.target.value)}
        onKeyDown={(event) => { if (event.key === "Enter") commitTitle(); }} />
      <button type="button" onClick={commitTitle}><Bookmark size={15} />保存名称</button>
    </label> : <button onClick={() => setRenaming(true)}><Pencil size={17} />重命名</button>}
    <button onClick={onMove}><Move size={17} />进入移动状态</button>
    <div className="context-menu-separator" />
    <button className="danger" onClick={onDelete}><Trash2 size={17} />删除书签</button>
  </div>;
}
