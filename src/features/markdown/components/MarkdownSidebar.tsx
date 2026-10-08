import { Files, ListTree, Network, Pin, Search, Settings } from "lucide-react";
import { useEffect, useRef, useState, type PointerEvent, type ReactNode } from "react";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import { MarkdownOutline } from "./MarkdownOutline";
import { DocumentFileNavigator } from "../../documents";
import { DocumentSearchPanel } from "../../search";
import { useMarkdownSearch } from "../search";
import { useSettings } from "../../settings";

type SidebarTab = "search" | "files" | "outline";

const tabLabels: Record<SidebarTab, string> = { search: "搜索", files: "文件", outline: "目录" };

export function MarkdownSidebar({ onResizeStart }: { onResizeStart?: (event: PointerEvent) => void }) {
  const workspace = useMarkdownWorkspace();
  const search = useMarkdownSearch();
  const settings = useSettings();
  const [tab, setTab] = useState<SidebarTab>("files");
  const [pinned, setPinned] = useState(false);
  const sidebarRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (workspace.sidebarCollapsed || pinned) return;
    const collapseOutside = (event: globalThis.PointerEvent) => {
      if (!sidebarRef.current?.contains(event.target as Node)) workspace.toggleSidebar();
    };
    window.addEventListener("pointerdown", collapseOutside);
    return () => window.removeEventListener("pointerdown", collapseOutside);
  }, [pinned, workspace.sidebarCollapsed, workspace.toggleSidebar]);

  function selectTab(nextTab: SidebarTab) {
    if (!workspace.sidebarCollapsed && tab === nextTab) {
      workspace.toggleSidebar();
      return;
    }
    setTab(nextTab);
    if (workspace.sidebarCollapsed) workspace.toggleSidebar();
  }

  return (
    <aside ref={sidebarRef} className={`markdown-sidebar document-navigation-sidebar ${workspace.sidebarCollapsed ? "collapsed" : ""}`} aria-label="文档导航">
      <nav className="document-activity-bar" aria-label="Markdown 功能">
        <ActivityButton tab="search" active={tab === "search" && !workspace.sidebarCollapsed} label="搜索" onSelect={selectTab}><Search size={20} /></ActivityButton>
        <ActivityButton tab="files" active={tab === "files" && !workspace.sidebarCollapsed} label="文件" onSelect={selectTab}><Files size={20} /></ActivityButton>
        <ActivityButton tab="outline" active={tab === "outline" && !workspace.sidebarCollapsed} label="目录" onSelect={selectTab}><ListTree size={20} /></ActivityButton>
        <DisabledActivityButton label="知识图谱（待开发）"><Network size={20} /></DisabledActivityButton>
        <button type="button" className="document-activity-button document-activity-settings"
          onClick={settings.openPanel} aria-label="设置中心" data-tooltip="设置中心"><Settings size={20} /></button>
      </nav>
      {!workspace.sidebarCollapsed && (
        <section className="document-feature-panel">
          <header className="document-feature-heading">
            <h2>{tabLabels[tab]}</h2>
            <button type="button" className={pinned ? "active" : ""} onClick={() => setPinned((value) => !value)}
              data-tooltip={pinned ? "取消置顶" : "置顶功能面板"} aria-label={pinned ? "取消置顶功能面板" : "置顶功能面板"}
              aria-pressed={pinned}><Pin size={17} fill={pinned ? "currentColor" : "none"} /></button>
          </header>
          <div className="markdown-sidebar-content document-feature-content">
            {tab === "search" && <DocumentSearchPanel search={search} placeholder="搜索当前 Markdown" />}
            {tab !== "search" && !workspace.activeDocument && <div className="markdown-sidebar-empty">打开 Markdown 后显示导航</div>}
            {workspace.activeDocument && tab === "files" && (
              <DocumentFileNavigator />
            )}
            {workspace.activeDocument && tab === "outline" && (
              <MarkdownOutline
                items={workspace.outline}
                activeId={workspace.activeOutlineId}
                onNavigate={workspace.navigateToOutline}
              />
            )}
          </div>
        </section>
      )}
      {!workspace.sidebarCollapsed && onResizeStart && (
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

function ActivityButton({ tab, active, label, onSelect, children }: {
  tab: SidebarTab; active: boolean; label: string; onSelect: (tab: SidebarTab) => void; children: ReactNode;
}) {
  return <button type="button" className={`document-activity-button ${active ? "active" : ""}`}
    onClick={() => onSelect(tab)} aria-label={label} aria-pressed={active} data-tooltip={label}>{children}</button>;
}
