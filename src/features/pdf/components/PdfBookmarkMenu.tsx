import { useState } from "react";
import { BookmarkPlus } from "lucide-react";
import type { PdfBookmarkDraft } from "../types/pdfBookmark";

export function PdfBookmarkMenu({ menuX, menuY, bookmark, onAdd }: {
  menuX: number;
  menuY: number;
  bookmark: PdfBookmarkDraft;
  onAdd: (bookmark: PdfBookmarkDraft) => void;
}) {
  const [title, setTitle] = useState("");
  const submit = () => onAdd({ ...bookmark, title: title.trim() || undefined });
  return <div className="selection-context-menu bookmark-context-menu" style={{ left: menuX, top: menuY }}
    onPointerDown={(event) => event.stopPropagation()}>
    <label className="bookmark-name-field"><span>书签名称</span>
      <input autoFocus value={title} maxLength={80} placeholder={`第 ${bookmark.page} 页书签`}
        onChange={(event) => setTitle(event.target.value)}
        onKeyDown={(event) => { if (event.key === "Enter") submit(); }} />
    </label>
    <button onClick={submit}><BookmarkPlus size={17} />添加书签</button>
  </div>;
}
