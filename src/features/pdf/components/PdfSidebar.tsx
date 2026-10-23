import { useEffect, useRef, useState, type PointerEvent, type ReactNode } from "react";
import { Bookmark, Files, FolderOpen, GalleryVerticalEnd, ListTree, Network, NotebookPen, Pin, Search, Settings } from "lucide-react";
import { usePdfOutline } from "../hooks/usePdfOutline";
import { usePdfSession } from "../hooks/usePdfSession";
import { PdfOutlineTree } from "./PdfOutlineTree";
import { PdfThumbnailList } from "./PdfThumbnailList";
import type { PdfBookmark } from "../types/pdfBookmark";
import { DocumentFileNavigator } from "../../documents";
import { DocumentSearchPanel } from "../../search";
import { usePdfSearch } from "../search";
import { PdfBookmarkPanel } from "./PdfBookmarkPanel";
import { useSettings } from "../../settings";
import { AnnotationNotesPanel } from "../../annotations";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";

type SidebarTab = "search" | "bookmarks" | "files" | "pages" | "outline" | "notes";

const tabLabels: Record<SidebarTab, string> = { search: "搜索", bookmarks: "书签", files: "文件", pages: "缩略图", outline: "目录", notes: "笔记" };

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
  const search = usePdfSearch();
  const settings = useSettings();
  const workspace = usePdfWorkspace();
  const [tab, setTab] = useState<SidebarTab>("pages");
  const [pinned, setPinned] = useState(false);
  const sidebarRef = useRef<HTMLElement | null>(null);
  const outline = usePdfOutline(document, currentPage);

  useEffect(() => {
    if (collapsed || pinned) return;
    const collapseOutside = (event: globalThis.PointerEvent) => {
      if (!sidebarRef.current?.contains(event.target as Node)) onToggleCollapsed();
    };
    window.addEventListener("pointerdown", collapseOutside);
    return () => window.removeEventListener("pointerdown", collapseOutside);
  }, [collapsed, pinned, onToggleCollapsed]);

  function selectTab(nextTab: SidebarTab) {
    if (!collapsed && tab === nextTab) {
      onToggleCollapsed();
      return;
    }
    setTab(nextTab);
    if (collapsed) onToggleCollapsed();
  }

  return (
    <aside ref={sidebarRef} className={`pdf-sidebar document-navigation-sidebar ${collapsed ? "collapsed" : ""}`} aria-label="文档导航">
      <nav className="document-activity-bar" aria-label="PDF 功能">
        <DisabledActivityButton label="页面（待开发）"><GalleryVerticalEnd size={20} /></DisabledActivityButton>
        <ActivityButton tab="search" active={tab === "search" && !collapsed} label="搜索" onSelect={selectTab}><Search size={20} /></ActivityButton>
        <ActivityButton tab="files" active={tab === "files" && !collapsed} label="文件" onSelect={selectTab}><FolderOpen size={20} /></ActivityButton>
        <ActivityButton tab="pages" active={tab === "pages" && !collapsed} label="缩略图" onSelect={selectTab}><Files size={20} /></ActivityButton>
        <ActivityButton tab="outline" active={tab === "outline" && !collapsed} label="目录" onSelect={selectTab}><ListTree size={20} /></ActivityButton>
        <ActivityButton tab="notes" active={tab === "notes" && !collapsed} label="笔记" count={workspace.annotations.length} onSelect={selectTab}><NotebookPen size={20} /></ActivityButton>
        <ActivityButton tab="bookmarks" active={tab === "bookmarks" && !collapsed} label="书签" count={bookmarks.length} onSelect={selectTab}><Bookmark size={20} /></ActivityButton>
        <DisabledActivityButton label="知识图谱（待开发）"><Network size={20} /></DisabledActivityButton>
        <button type="button" className="document-activity-button document-activity-settings"
          onClick={settings.openPanel} aria-label="设置中心" data-tooltip="设置中心"><Settings size={20} /></button>
      </nav>
      {!collapsed && (
        <section className="document-feature-panel">
          <header className="document-feature-heading">
            <h2>{tabLabels[tab]}</h2>
            <button type="button" className={pinned ? "active" : ""} onClick={() => setPinned((value) => !value)}
              data-tooltip={pinned ? "取消置顶" : "置顶功能面板"} aria-label={pinned ? "取消置顶功能面板" : "置顶功能面板"}
              aria-pressed={pinned}><Pin size={17} fill={pinned ? "currentColor" : "none"} /></button>
          </header>
          <div className="sidebar-content document-feature-content">
            {tab === "search" && <DocumentSearchPanel search={search} placeholder="搜索当前 PDF" />}
            {tab === "notes" && <AnnotationNotesPanel annotations={workspace.annotations} onUpdate={workspace.updateAnnotation}
              onDelete={workspace.deleteAnnotation} onMerge={workspace.mergeAnnotations}
              selectedAnnotationId={workspace.selectedAnnotationId} onSelect={workspace.navigateToAnnotation} />}
            {tab === "bookmarks" && <PdfBookmarkPanel />}
            {tab !== "search" && tab !== "notes" && tab !== "bookmarks" && !document && <div className="sidebar-empty">打开 PDF 后显示导航</div>}
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
        </section>
      )}
      {!collapsed && onResizeStart && (
        <div className="panel-resize-handle right-edge" role="separator" aria-orientation="vertical"
          aria-label="调整文档导航宽度" onPointerDown={onResizeStart} />
      )}
    </aside>
  );
}

function DisabledActivityButton({ label, children }: { label: string; children: ReactNode }) {
  return <button type="button" className="document-activity-button is-disabled" aria-disabled="true"
    aria-label={label} data-tooltip={label} title={label}>{children}</button>;
}

function ActivityButton({ tab, active, label, count, onSelect, children }: {
  tab: SidebarTab; active: boolean; label: string; count?: number; onSelect: (tab: SidebarTab) => void; children: ReactNode;
}) {
  return <button type="button" className={`document-activity-button ${active ? "active" : ""}`}
    onClick={() => onSelect(tab)} aria-label={label} aria-pressed={active} data-tooltip={label}>
    {children}{Boolean(count) && <small className="document-activity-count">{count! > 99 ? "99+" : count}</small>}
  </button>;
}

function countBookmarksByPage(bookmarks: PdfBookmark[]) {
  const counts = new Map<number, number>();
  bookmarks.forEach((bookmark) => counts.set(bookmark.page, (counts.get(bookmark.page) ?? 0) + 1));
  return counts;
}
