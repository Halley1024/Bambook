import { Files, ListTree, PanelLeftClose, PanelLeftOpen } from "lucide-react";
import { useState, type PointerEvent } from "react";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import { MarkdownOutline } from "./MarkdownOutline";
import { DocumentFileNavigator } from "../../documents";

type SidebarTab = "files" | "outline";

export function MarkdownSidebar({ onResizeStart }: { onResizeStart?: (event: PointerEvent) => void }) {
  const workspace = useMarkdownWorkspace();
  const [tab, setTab] = useState<SidebarTab>("files");

  return (
    <aside className={`markdown-sidebar ${workspace.sidebarCollapsed ? "collapsed" : ""}`} aria-label="文档导航">
      {workspace.sidebarCollapsed ? (
        <button className="markdown-sidebar-expand" onClick={workspace.toggleSidebar} data-tooltip="展开文档导航：显示 Markdown 文件与目录">
          <PanelLeftOpen size={19} />
          <span>文档导航</span>
        </button>
      ) : (
        <>
          <header className="markdown-navigation-heading">
            <h2>文档导航</h2>
            <button className="markdown-sidebar-collapse" onClick={workspace.toggleSidebar} data-tooltip="收起文档导航">
              <PanelLeftClose size={18} />
            </button>
          </header>
          <div className="markdown-sidebar-tabs" role="tablist" aria-label="Markdown 导航方式">
            <button className={tab === "files" ? "active" : ""} data-tooltip="文件：浏览当前 Markdown 所在目录"
              onClick={() => setTab("files")} role="tab">
              <Files size={16} /><span className="markdown-sidebar-tab-label">文件</span>
            </button>
            <button className={tab === "outline" ? "active" : ""} data-tooltip="目录：浏览并跳转文档标题"
              onClick={() => setTab("outline")} role="tab">
              <ListTree size={16} /><span className="markdown-sidebar-tab-label">目录</span>
            </button>
          </div>
          <div className="markdown-sidebar-content">
            {!workspace.activeDocument && <div className="markdown-sidebar-empty">打开 Markdown 后显示导航</div>}
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
        </>
      )}
      {!workspace.sidebarCollapsed && onResizeStart && (
        <div className="panel-resize-handle right-edge" role="separator" aria-orientation="vertical"
          aria-label="调整文档导航宽度" onPointerDown={onResizeStart} />
      )}
    </aside>
  );
}
