import { useEffect, useRef, useState } from "react";
import { Bookmark, ChevronDown, ChevronLeft, ChevronRight, Eraser, Hand, Minus,
  MousePointer2, MoveHorizontal, Palette, Plus, Redo2, RotateCcw, RotateCw, Scan, SquareDashedMousePointer, Trash2, Undo2 } from "lucide-react";
import type { MarkupType } from "../../annotations";
import { useSettings } from "../../settings";
import { DocumentSearchTools } from "../../search";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";
import { MarkupIcon } from "./MarkupIcon";
import { usePdfSearch } from "../search";
import { PdfExistingBookmarkMenu } from "./PdfExistingBookmarkMenu";
import { BookmarkClearConfirmationDialog } from "./BookmarkClearConfirmationDialog";
import { ANNOTATION_CONTEXT_COLORS, ContextColorPicker } from "./ContextColorPicker";

const ZOOM_LEVELS = [50, 75, 100, 125, 150, 175, 200, 250];

export function PdfToolbar() {
  const workspace = usePdfWorkspace();
  const search = usePdfSearch();
  const { settings } = useSettings();
  const pageCount = workspace.activeDocument?.pageCount ?? 0;
  const [pageValue, setPageValue] = useState(String(workspace.currentPage));
  const [openPopover, setOpenPopover] = useState<"zoom" | "color" | "bookmarks" | null>(null);
  const [bookmarkMenu, setBookmarkMenu] = useState<{ id: string; x: number; y: number } | null>(null);
  const [pendingClear, setPendingClear] = useState<{ scope: "page" | "all"; count: number; page: number } | null>(null);
  const toolbarRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => setPageValue(String(workspace.currentPage)), [workspace.currentPage]);
  useEffect(() => {
    if (!openPopover) return;
    const close = (event: PointerEvent) => {
      if (!toolbarRef.current?.contains(event.target as Node)) setOpenPopover(null);
    };
    window.addEventListener("pointerdown", close);
    return () => window.removeEventListener("pointerdown", close);
  }, [openPopover]);

  function commitPage() {
    const requested = Number.parseInt(pageValue, 10);
    if (!Number.isFinite(requested) || pageCount === 0) {
      setPageValue(String(workspace.currentPage));
      return;
    }
    const page = Math.max(1, Math.min(pageCount, requested));
    setPageValue(String(page));
    if (page !== workspace.currentPage) workspace.navigateToPage(page);
  }

  function requestClearBookmarks(scope: "page" | "all") {
    const count = scope === "page"
      ? workspace.bookmarks.filter((bookmark) => bookmark.page === workspace.currentPage).length
      : workspace.bookmarks.length;
    if (!count) return;
    if (settings.confirmationRemindersEnabled) { setPendingClear({ scope, count, page: workspace.currentPage }); return; }
    workspace.clearBookmarks(scope === "page" ? workspace.currentPage : undefined);
  }

  function confirmClearBookmarks() {
    if (!pendingClear) return;
    workspace.clearBookmarks(pendingClear.scope === "page" ? pendingClear.page : undefined);
    setPendingClear(null);
  }

  return <div className="mode-toolbar pdf-mode-toolbar" aria-label="PDF 工具" ref={toolbarRef}>
    <div className="toolbar-group pdf-page-navigation" aria-label="PDF 页面与阅读历史">
      <button className="icon-button" data-tooltip="上一页：跳转到前一页" aria-label="上一页" disabled={!workspace.activeDocument || workspace.currentPage <= 1}
        onClick={() => workspace.navigateToPage(workspace.currentPage - 1)}><ChevronLeft size={19} /></button>
      <label className="pdf-page-input" title="输入页码并按回车跳转">
        <input value={pageValue} inputMode="numeric" aria-label="当前页码"
          onChange={(event) => setPageValue(event.target.value.replace(/[^0-9]/g, ""))} onBlur={commitPage}
          onKeyDown={(event) => {
            if (event.key === "Enter") { commitPage(); event.currentTarget.blur(); }
            if (event.key === "Escape") { setPageValue(String(workspace.currentPage)); event.currentTarget.blur(); }
          }} />
        <span>/ {pageCount}</span>
      </label>
      <button className="icon-button" data-tooltip="下一页：跳转到后一页" aria-label="下一页" disabled={!workspace.activeDocument || workspace.currentPage >= pageCount}
        onClick={() => workspace.navigateToPage(workspace.currentPage + 1)}><ChevronRight size={19} /></button>
      <button className="icon-button" data-tooltip="历史后退：返回上一个阅读位置" aria-label="阅读历史后退"
        disabled={!workspace.canNavigateHistoryBack} onClick={workspace.navigateHistoryBack}><Undo2 size={18} /></button>
      <button className="icon-button" data-tooltip="历史前进：前往下一个阅读位置" aria-label="阅读历史前进"
        disabled={!workspace.canNavigateHistoryForward} onClick={workspace.navigateHistoryForward}><Redo2 size={18} /></button>
    </div>

    <div className="toolbar-group pdf-zoom-tools" aria-label="PDF 缩放">
      <button className="icon-button" onClick={() => workspace.setScale(Math.max(0.5, workspace.scale - 0.1))}
        data-tooltip="缩小：降低当前页面比例" aria-label="缩小"><Minus size={18} /></button>
      <div className="toolbar-popover-host">
        <button className={`pdf-scale-trigger ${openPopover === "zoom" ? "active" : ""}`}
          onClick={() => setOpenPopover(openPopover === "zoom" ? null : "zoom")}
          data-tooltip="缩放比例：选择页面显示大小" aria-haspopup="menu" aria-expanded={openPopover === "zoom"}>
          {Math.round(workspace.scale * 100)}% <ChevronDown size={14} />
        </button>
        {openPopover === "zoom" && <div className="toolbar-popover zoom-level-menu" role="menu">
          {ZOOM_LEVELS.map((level) => <button key={level} role="menuitemradio"
            data-tooltip={`缩放：将页面设置为 ${level}%`}
            aria-checked={Math.round(workspace.scale * 100) === level}
            onClick={() => { workspace.setScale(level / 100); setOpenPopover(null); }}>{level}%</button>)}
        </div>}
      </div>
      <button className="icon-button" onClick={() => workspace.setScale(Math.min(2.5, workspace.scale + 0.1))}
        data-tooltip="放大：提高当前页面比例" aria-label="放大"><Plus size={18} /></button>
      <button className={`icon-button fit-view-button ${workspace.zoomMode === "fit-width" ? "active" : ""}`}
        onClick={workspace.fitToWidth} data-tooltip="适合宽度：页面宽度匹配阅读区" aria-label="适合宽度"><MoveHorizontal size={18} /></button>
      <button className={`icon-button fit-view-button ${workspace.zoomMode === "fit-page" ? "active" : ""}`}
        onClick={workspace.fitToPage} data-tooltip="适合页面：完整显示当前页面" aria-label="适合页面"><Scan size={18} /></button>
    </div>

    <DocumentSearchTools search={search} placeholder="搜索 PDF 文档" clearOnClose />

    <div className="toolbar-group pdf-annotation-tools" aria-label="PDF 文字批注工具">
      <button className="tool-button annotation-history-button" disabled={!workspace.canUndoAnnotations}
        onClick={workspace.undoAnnotations} data-tooltip="撤销批注：撤销最近一次修改（Ctrl+Z）">
        <RotateCcw size={17} /><span className="tool-label">撤销批注</span>
      </button>
      <button className="tool-button annotation-history-button" disabled={!workspace.canRedoAnnotations}
        onClick={workspace.redoAnnotations} data-tooltip="重做批注：恢复刚撤销的修改（Ctrl+Y）">
        <RotateCw size={17} /><span className="tool-label">重做批注</span>
      </button>
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "pointer" ? "active" : ""}`}
        onClick={workspace.selectPointerTool} data-tooltip="点击：选择文字或操作页面内容" aria-label="点击工具">
        <MousePointer2 size={17} /><span className="tool-label">点击</span>
      </button>
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "pan" ? "active" : ""}`}
        onClick={workspace.selectPanTool} data-tooltip="拖拽：按住鼠标移动 PDF 页面" aria-label="拖拽工具">
        <Hand size={17} /><span className="tool-label">拖拽</span>
      </button>
      {(["underline", "squiggly", "strikeout", "highlight"] as MarkupType[]).map((type) => <button key={type}
        className={`tool-button markup-tool-button ${workspace.interactionMode === "markup" && workspace.activeMarkupType === type ? "active" : ""}`}
        onClick={() => workspace.selectMarkupTool(type)} data-tooltip={`${markupLabel(type)}：选中文字后自动添加样式`} aria-label={markupLabel(type)}>
        <MarkupIcon type={type} size={17} /><span className="tool-label">{markupLabel(type)}</span>
      </button>)}
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "area" ? "active" : ""}`}
        onClick={workspace.selectAreaTool} data-tooltip="选择区域：在 PDF 页面拖动绘制矩形区域" aria-label="选择区域工具">
        <SquareDashedMousePointer size={17} /><span className="tool-label">选择区域</span>
      </button>
      <div className="toolbar-popover-host">
        <button className={`tool-button annotation-palette-button ${openPopover === "color" ? "active" : ""}`}
          onClick={() => setOpenPopover(openPopover === "color" ? null : "color")}
          data-tooltip="调色盘：设置新批注或当前批注的颜色" aria-label="调色盘" aria-haspopup="dialog" aria-expanded={openPopover === "color"}>
          <Palette className="annotation-palette-icon" size={18} style={{ color: workspace.activeAnnotationColor }} />
          <span className="tool-label">调色盘</span>
        </button>
        {openPopover === "color" && <div className="toolbar-popover annotation-toolbar-color-popover">
          <ContextColorPicker value={workspace.activeAnnotationColor} onChange={workspace.setAnnotationColor}
            scopeLabel="批注" colors={ANNOTATION_CONTEXT_COLORS} />
        </div>}
      </div>
      <button className={`tool-button annotation-mode-button ${workspace.interactionMode === "eraser" ? "active" : ""}`}
        onClick={workspace.selectEraserTool} data-tooltip="橡皮：滑过批注以删除样式和笔记" aria-label="橡皮工具">
        <Eraser size={17} /><span className="tool-label">橡皮</span>
      </button>
    </div>

    <div className="toolbar-popover-host pdf-bookmark-tools">
      <button className={`tool-button ${openPopover === "bookmarks" ? "active" : ""}`}
        onClick={() => setOpenPopover(openPopover === "bookmarks" ? null : "bookmarks")}
        data-tooltip="书签栏：查看和跳转当前文档书签" aria-haspopup="dialog" aria-expanded={openPopover === "bookmarks"}>
        <Bookmark size={17} /><span>书签栏</span>
        {workspace.bookmarks.length > 0 && <span className="toolbar-count-badge">{workspace.bookmarks.length}</span>}
      </button>
      {openPopover === "bookmarks" && <div className="toolbar-popover bookmark-list-popover" role="dialog" aria-label="书签栏">
        <div className="bookmark-list-heading"><strong>书签</strong><span>{workspace.bookmarks.length}</span></div>
        {workspace.bookmarks.length === 0 ? <p>在 PDF 页面空白处右键添加书签。</p> : <div className="bookmark-list">
          {[...workspace.bookmarks].sort((a, b) => a.page - b.page || a.y - b.y || a.x - b.x).map((bookmark) => <div className="bookmark-list-item" key={bookmark.id}
            onContextMenu={(event) => {
              event.preventDefault();
              setBookmarkMenu({ id: bookmark.id, x: event.clientX, y: event.clientY });
            }}>
            <button className="bookmark-jump" data-tooltip={bookmark.title}
              onClick={() => { workspace.navigateToBookmark(bookmark.id); setOpenPopover(null); }}>
              <Bookmark size={15} fill="currentColor" style={{ color: bookmark.color }} /><span>{bookmark.title}</span><small>第 {bookmark.page} 页</small>
            </button>
            <button className="bookmark-delete" onClick={() => workspace.deleteBookmark(bookmark.id)} data-tooltip="删除书签：移除这条位置记录"
              aria-label={`删除${bookmark.title}`}><Trash2 size={15} /></button>
          </div>)}
        </div>}
        <div className="bookmark-list-actions">
          <button disabled={!workspace.bookmarks.some((bookmark) => bookmark.page === workspace.currentPage)}
            onClick={() => requestClearBookmarks("page")}>清除当前页</button>
          <button className="danger" disabled={!workspace.bookmarks.length}
            onClick={() => requestClearBookmarks("all")}>清除全部</button>
        </div>
      </div>}
    </div>

    {bookmarkMenu && (() => {
      const bookmark = workspace.bookmarks.find((item) => item.id === bookmarkMenu.id);
      return bookmark ? <PdfExistingBookmarkMenu x={bookmarkMenu.x} y={bookmarkMenu.y} bookmark={bookmark}
        onUpdate={(changes) => workspace.updateBookmark(bookmark.id, changes)}
        onMove={() => { workspace.navigateToBookmark(bookmark.id); workspace.beginBookmarkMove(bookmark.id); setBookmarkMenu(null); setOpenPopover(null); }}
        onDelete={() => { workspace.deleteBookmark(bookmark.id); setBookmarkMenu(null); }}
        onClose={() => setBookmarkMenu(null)} /> : null;
    })()}

    {pendingClear && <BookmarkClearConfirmationDialog
      description={pendingClear.scope === "page"
        ? `确定清除第 ${pendingClear.page} 页的 ${pendingClear.count} 个书签吗？此操作将在保存文档后写入存储。`
        : `确定清除当前文档的全部 ${pendingClear.count} 个书签吗？此操作将在保存文档后写入存储。`}
      onCancel={() => setPendingClear(null)} onConfirm={confirmClearBookmarks} />}

  </div>;
}

function markupLabel(type: MarkupType) {
  if (type === "highlight") return "高亮";
  if (type === "underline") return "下划线";
  if (type === "squiggly") return "波浪线";
  return "删除线";
}
