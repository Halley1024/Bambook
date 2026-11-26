import type { PointerEvent, ReactNode } from "react";
import { PanelRightClose } from "lucide-react";

export type WorkspaceTool = { id: string; label: string; icon: ReactNode; disabled?: boolean };

export function WorkspaceTools({ tools, active, collapsed, onSelect, onToggleCollapsed, onResizeStart, children }: {
  tools: WorkspaceTool[];
  active: string;
  collapsed: boolean;
  onSelect: (id: string) => void;
  onToggleCollapsed: () => void;
  onResizeStart?: (event: PointerEvent) => void;
  children: ReactNode;
}) {
  return <aside className={`workspace-tools ${collapsed ? "collapsed" : ""}`} aria-label="文档工具">
    <section className="workspace-tool-panel" hidden={collapsed}>
      <header className="document-feature-heading">
        <h2>{tools.find((tool) => tool.id === active)?.label}</h2>
        <button type="button" onClick={onToggleCollapsed} aria-label="收起工具面板" data-tooltip="收起工具面板"><PanelRightClose size={18} /></button>
      </header>
      <div className="workspace-tool-content">{children}</div>
    </section>
    <nav className="document-activity-bar" aria-label="文档工具切换">
      {tools.map((tool) => <button key={tool.id} type="button"
        className={`document-activity-button ${!collapsed && active === tool.id ? "active" : ""}`}
        aria-label={tool.label} data-tooltip={tool.label} disabled={tool.disabled}
        aria-pressed={!collapsed && active === tool.id}
        onClick={() => { onSelect(tool.id); if (collapsed || active === tool.id) onToggleCollapsed(); }}>
        {tool.icon}
      </button>)}
    </nav>
    {!collapsed && onResizeStart && <div className="panel-resize-handle left-edge" role="separator" aria-orientation="vertical" aria-label="调整工具面板宽度" onPointerDown={onResizeStart} />}
  </aside>;
}
