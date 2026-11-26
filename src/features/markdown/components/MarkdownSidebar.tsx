import type { AssistantActionRequest } from "../../../components/ai/assistantActions";
import { Bot, ListTree, Search } from "lucide-react";
import { useEffect, useState, type PointerEvent } from "react";
import { WorkspaceTools } from "../../../components/navigation/WorkspaceTools";
import { AssistantSidebar } from "../../../components/ai/AssistantSidebar";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import { MarkdownOutline } from "./MarkdownOutline";
import { DocumentSearchPanel } from "../../search";
import { useMarkdownSearch } from "../search";

export function MarkdownSidebar({ onResizeStart, assistantRequest }: { onResizeStart?: (event: PointerEvent) => void; assistantRequest?: AssistantActionRequest | null }) {
  const workspace = useMarkdownWorkspace();
  const search = useMarkdownSearch();
  const [tab, setTab] = useState("outline");
  useEffect(() => { if (assistantRequest) setTab("assistant"); }, [assistantRequest]);
  const selectedText = workspace.content.slice(workspace.selection.start, workspace.selection.end).trim();
  return <WorkspaceTools tools={[
    { id: "search", label: "搜索", icon: <Search size={20} /> },
    { id: "outline", label: "目录", icon: <ListTree size={20} /> },
    { id: "assistant", label: "AI 小助手", icon: <Bot size={20} /> },
  ]} active={tab} collapsed={workspace.helpPanelCollapsed} onSelect={setTab} onToggleCollapsed={workspace.toggleHelpPanel} onResizeStart={onResizeStart}>
    <div className="workspace-feature-body" hidden={tab === "assistant"}>
      {tab === "search" && <DocumentSearchPanel search={search} placeholder="搜索当前 Markdown" />}
      {tab === "outline" && <MarkdownOutline items={workspace.outline} activeId={workspace.activeOutlineId} onNavigate={workspace.navigateToOutline} />}
    </div>
    <div className="workspace-assistant-host" hidden={tab !== "assistant"}>
      <AssistantSidebar embedded actionRequest={assistantRequest} contextText={selectedText} contextLabel="Markdown 选中文本" emptyContextHint="在 Markdown 编辑器中选择一段文字以带入上下文。" collapsed={false} onToggleCollapsed={workspace.toggleHelpPanel} />
    </div>
  </WorkspaceTools>;
}
