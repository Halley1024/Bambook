import { Bookmark, Move, Trash2 } from "lucide-react";
import { useState } from "react";
import { useSettings } from "../../settings";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";
import { BookmarkClearConfirmationDialog } from "./BookmarkClearConfirmationDialog";
import { PdfExistingBookmarkMenu } from "./PdfExistingBookmarkMenu";

export function PdfBookmarkPanel() {
  const workspace = usePdfWorkspace();
  const { settings } = useSettings();
  const [bookmarkMenu, setBookmarkMenu] = useState<{ id: string; x: number; y: number } | null>(null);
  const [pendingClear, setPendingClear] = useState<{ scope: "page" | "all"; count: number; page: number } | null>(null);
  const bookmarks = [...workspace.bookmarks].sort((a, b) => a.page - b.page || a.y - b.y || a.x - b.x);

  function requestClear(scope: "page" | "all") {
    const count = scope === "page"
      ? workspace.bookmarks.filter((bookmark) => bookmark.page === workspace.currentPage).length
      : workspace.bookmarks.length;
    if (!count) return;
    if (settings.confirmationRemindersEnabled) {
      setPendingClear({ scope, count, page: workspace.currentPage });
      return;
    }
    workspace.clearBookmarks(scope === "page" ? workspace.currentPage : undefined);
  }

  function confirmClear() {
    if (!pendingClear) return;
    workspace.clearBookmarks(pendingClear.scope === "page" ? pendingClear.page : undefined);
    setPendingClear(null);
  }

  return (
    <div className="pdf-sidebar-bookmarks">
      <div className="pdf-sidebar-bookmark-summary">
        <span><Bookmark size={15} />共 {bookmarks.length} 个书签</span>
        <small>右键书签可重命名、换色或移动</small>
      </div>

      {bookmarks.length === 0 ? (
        <div className="pdf-sidebar-bookmark-empty">
          <Bookmark size={28} />
          <span>暂无书签</span>
          <small>在 PDF 页面空白处或选中文字后右键添加。</small>
        </div>
      ) : (
        <div className="pdf-sidebar-bookmark-list">
          {bookmarks.map((bookmark) => (
            <div className="pdf-sidebar-bookmark-item" key={bookmark.id}
              onContextMenu={(event) => {
                event.preventDefault();
                setBookmarkMenu({ id: bookmark.id, x: event.clientX, y: event.clientY });
              }}>
              <button type="button" className="pdf-sidebar-bookmark-jump" onClick={() => workspace.navigateToBookmark(bookmark.id)}>
                <Bookmark size={16} fill="currentColor" style={{ color: bookmark.color }} />
                <span><strong>{bookmark.title}</strong><small>第 {bookmark.page} 页</small></span>
              </button>
              <button type="button" className="pdf-sidebar-bookmark-move"
                onClick={() => { workspace.navigateToBookmark(bookmark.id); workspace.beginBookmarkMove(bookmark.id); }}
                aria-label={`移动${bookmark.title}`} data-tooltip="移动书签"><Move size={15} /></button>
              <button type="button" className="pdf-sidebar-bookmark-delete" onClick={() => workspace.deleteBookmark(bookmark.id)}
                aria-label={`删除${bookmark.title}`} data-tooltip="删除书签"><Trash2 size={15} /></button>
            </div>
          ))}
        </div>
      )}

      <div className="pdf-sidebar-bookmark-actions">
        <button type="button" disabled={!workspace.bookmarks.some((bookmark) => bookmark.page === workspace.currentPage)}
          onClick={() => requestClear("page")}>清除当前页</button>
        <button type="button" className="danger" disabled={!workspace.bookmarks.length}
          onClick={() => requestClear("all")}>清除全部</button>
      </div>

      {bookmarkMenu && (() => {
        const bookmark = workspace.bookmarks.find((item) => item.id === bookmarkMenu.id);
        return bookmark ? (
          <PdfExistingBookmarkMenu x={bookmarkMenu.x} y={bookmarkMenu.y} bookmark={bookmark}
            onUpdate={(changes) => workspace.updateBookmark(bookmark.id, changes)}
            onMove={() => { workspace.navigateToBookmark(bookmark.id); workspace.beginBookmarkMove(bookmark.id); setBookmarkMenu(null); }}
            onDelete={() => { workspace.deleteBookmark(bookmark.id); setBookmarkMenu(null); }}
            onClose={() => setBookmarkMenu(null)} />
        ) : null;
      })()}

      {pendingClear && (
        <BookmarkClearConfirmationDialog
          description={pendingClear.scope === "page"
            ? `确定清除第 ${pendingClear.page} 页的 ${pendingClear.count} 个书签吗？此操作将在保存文档后写入存储。`
            : `确定清除当前文档的全部 ${pendingClear.count} 个书签吗？此操作将在保存文档后写入存储。`}
          onCancel={() => setPendingClear(null)} onConfirm={confirmClear} />
      )}
    </div>
  );
}
