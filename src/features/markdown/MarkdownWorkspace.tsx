import type { CSSProperties } from "react";
import { AssistantSidebar } from "../../components/ai/AssistantSidebar";
import { MarkdownSidebar } from "./components/MarkdownSidebar";
import { MarkdownSplitView } from "./components/MarkdownSplitView";
import { MarkdownStatusBar } from "./components/MarkdownStatusBar";
import { useMarkdownWorkspace } from "./hooks/useMarkdownWorkspace";
import { useWorkspacePanelWidths } from "../../shared/useWorkspacePanelWidths";
import { useSettings } from "../settings";

export function MarkdownWorkspace() {
  const workspace = useMarkdownWorkspace();
  const { settings, updateWorkspaceLayout } = useSettings();
  const panels = useWorkspacePanelWidths(
    settings.markdownWorkspaceLayout,
    (layout) => updateWorkspaceLayout("markdown", layout),
    { onLeftCollapse: workspace.sidebarCollapsed ? undefined : workspace.toggleSidebar },
  );
  const panelStyle = {
    "--workspace-left-width": `${panels.leftWidth}px`,
    "--workspace-right-width": `${panels.rightWidth}px`,
  } as CSSProperties;
  const selectedText = workspace.content.slice(workspace.selection.start, workspace.selection.end).trim();

  const content = <section className="workspace-content">
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
      <AssistantSidebar
        contextText={selectedText}
        contextLabel="Markdown 选中文本"
        emptyContextHint="在 Markdown 编辑器中选择一段文字以带入上下文。"
        collapsed={workspace.helpPanelCollapsed}
        onToggleCollapsed={workspace.toggleHelpPanel}
        onResizeStart={panels.beginRightResize}
      />
    </main>
  );
}
