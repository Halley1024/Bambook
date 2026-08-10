import { useState, type PointerEvent } from "react";
import { Files, FolderOpen, ListTree, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { usePdfOutline } from "../hooks/usePdfOutline";
import { usePdfSession } from "../hooks/usePdfSession";
import { PdfOutlineTree } from "./PdfOutlineTree";
import { PdfThumbnailList } from "./PdfThumbnailList";
import type { PdfBookmark } from "../types/pdfBookmark";
import { DocumentFileNavigator } from "../../documents";

type SidebarTab = "files" | "pages" | "outline";

type PdfSidebarProps = {
  currentPage: number;
  onNavigatePage: (page: number) => void;
  onNavigateOutline: (page: number, title: string) => void;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onResizeStart?: (event: PointerEvent) => void;
  bookmarks: PdfBookmark[];
};

export function PdfSidebar({ currentPage, onNavigatePage, onNavigateOutline, collapsed, onToggleCollapsed, onResizeStart, bookmarks }: PdfSidebarProps) {
  const { document } = usePdfSession();
  const [tab, setTab] = useState<SidebarTab>("pages");
  const outline = usePdfOutline(document, currentPage);

  return (
    <aside className={`pdf-sidebar ${collapsed ? "collapsed" : ""}`} aria-label="文档导航">
      {collapsed ? (
        <button className="sidebar-expand-button" onClick={onToggleCollapsed} data-tooltip="展开文档导航：显示 PDF 文件、视图和目录">
          <PanelLeftOpen size={19} />
          <span>文档导航</span>
        </button>
      ) : (
        <>
          <header className="sidebar-heading">
            <h2>文档导航</h2>
            <button className="sidebar-collapse-button" onClick={onToggleCollapsed} data-tooltip="收起文档导航">
              <PanelLeftClose size={18} />
            </button>
          </header>
          <div className="sidebar-tabs" role="tablist" aria-label="PDF 导航方式">
            <button
              className={`sidebar-tab ${tab === "files" ? "active" : ""}`}
              data-tooltip="文件：查看当前 PDF 文件信息"
              role="tab"
              aria-selected={tab === "files"}
              onClick={() => setTab("files")}
            >
              <FolderOpen size={16} />
              <span className="sidebar-tab-label">文件</span>
            </button>
            <button
              className={`sidebar-tab ${tab === "pages" ? "active" : ""}`}
              data-tooltip="视图：浏览并跳转 PDF 页面"
              role="tab"
              aria-selected={tab === "pages"}
              onClick={() => setTab("pages")}
            >
              <Files size={16} />
              <span className="sidebar-tab-label">视图</span>
            </button>
            <button
              className={`sidebar-tab ${tab === "outline" ? "active" : ""}`}
              data-tooltip="目录：浏览并跳转 PDF 内置目录"
              role="tab"
              aria-selected={tab === "outline"}
              onClick={() => setTab("outline")}
            >
              <ListTree size={16} />
              <span className="sidebar-tab-label">目录</span>
            </button>
          </div>

          <div className="sidebar-content">
            {!document && <div className="sidebar-empty">打开 PDF 后显示导航</div>}
            {document && tab === "files" && (
              <DocumentFileNavigator />
            )}
            {document && tab === "pages" && (
              <PdfThumbnailList document={document} currentPage={currentPage} onNavigate={onNavigatePage}
                bookmarkCounts={countBookmarksByPage(bookmarks)} />
            )}
            {document && tab === "outline" && (
              <PdfOutlineTree
                items={outline.outline}
                loading={outline.loading}
                expanded={outline.expanded}
                activeId={outline.activeId}
                itemRefs={outline.itemRefs}
                onToggle={outline.toggleExpanded}
                onNavigate={onNavigateOutline}
              />
            )}
          </div>
        </>
      )}
      {!collapsed && onResizeStart && (
        <div className="panel-resize-handle right-edge" role="separator" aria-orientation="vertical"
          aria-label="调整文档导航宽度" onPointerDown={onResizeStart} />
      )}
    </aside>
  );
}

function countBookmarksByPage(bookmarks: PdfBookmark[]) {
  const counts = new Map<number, number>();
  bookmarks.forEach((bookmark) => counts.set(bookmark.page, (counts.get(bookmark.page) ?? 0) + 1));
  return counts;
}
