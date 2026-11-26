import { useState, type CSSProperties } from "react";
import { AssistantTextContextMenu } from "../../components/ai/AssistantTextContextMenu";
import type { AssistantActionRequest } from "../../components/ai/assistantActions";
import { MarkdownSidebar } from "./components/MarkdownSidebar";
import { MarkdownSplitView } from "./components/MarkdownSplitView";
import { useMarkdownWorkspace } from "./hooks/useMarkdownWorkspace";
import { useWorkspacePanelWidths } from "../../shared/useWorkspacePanelWidths";
import { useSettings } from "../settings";

export function MarkdownWorkspace() {
  const workspace = useMarkdownWorkspace();
  const [assistantRequest, setAssistantRequest] = useState<AssistantActionRequest | null>(null);
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
    <section className="reader-area markdown-reader-area">
      <AssistantTextContextMenu onRequest={(text, action) => {
        setAssistantRequest({ id: crypto.randomUUID(), text, action });
        if (workspace.helpPanelCollapsed) workspace.toggleHelpPanel();
      }}><MarkdownSplitView /></AssistantTextContextMenu>
    </section>
  </section>;

  return (
    <main
      className={`workspace markdown-workspace tools-workspace ${
        workspace.helpPanelCollapsed ? "help-collapsed" : ""
      } ${panels.resizing ? "resizing-panels" : ""}`}
      style={panelStyle}
    >
      {content}
      <MarkdownSidebar assistantRequest={assistantRequest} onResizeStart={panels.beginRightResize} />
    </main>
  );
}
