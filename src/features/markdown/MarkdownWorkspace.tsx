import type { CSSProperties, ReactNode } from "react";
import { MarkdownInspectorPanel } from "./components/MarkdownInspectorPanel";
import { MarkdownSidebar } from "./components/MarkdownSidebar";
import { MarkdownSplitView } from "./components/MarkdownSplitView";
import { MarkdownStatusBar } from "./components/MarkdownStatusBar";
import { useMarkdownWorkspace } from "./hooks/useMarkdownWorkspace";
import { useWorkspacePanelWidths } from "../../shared/useWorkspacePanelWidths";
import { useSettings } from "../settings";

type MarkdownWorkspaceProps = {
  documentTabs: ReactNode;
};

export function MarkdownWorkspace({ documentTabs }: MarkdownWorkspaceProps) {
  const workspace = useMarkdownWorkspace();
  const { settings, updateWorkspaceLayout } = useSettings();
  const panels = useWorkspacePanelWidths(
    settings.markdownWorkspaceLayout,
    (layout) => updateWorkspaceLayout("markdown", layout),
  );
  const panelStyle = {
    "--workspace-left-width": `${panels.leftWidth}px`,
    "--workspace-right-width": `${panels.rightWidth}px`,
  } as CSSProperties;

  const content = <section className="workspace-content">
    <div className="document-bar">{documentTabs}</div>
    <section className="reader-area markdown-reader-area">
      <MarkdownSplitView />
      <MarkdownStatusBar />
    </section>
  </section>;

  return (
    <main
      className={`workspace markdown-workspace ${workspace.sidebarCollapsed ? "sidebar-collapsed" : ""} ${
        workspace.helpPanelCollapsed ? "help-collapsed" : ""
      } ${panels.resizing ? "resizing-panels" : ""}`}
      style={panelStyle}
    >
      <MarkdownSidebar onResizeStart={panels.beginLeftResize} />
      {content}
      <MarkdownInspectorPanel onResizeStart={panels.beginRightResize} />
    </main>
  );
}
