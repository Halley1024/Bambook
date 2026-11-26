import type { AssistantActionRequest } from "../../../components/ai/assistantActions";
import { useEffect, useState, type PointerEvent } from "react";
import { Bookmark, Bot, Files, GalleryVerticalEnd, ListTree, NotebookPen, Search } from "lucide-react";
import { WorkspaceTools } from "../../../components/navigation/WorkspaceTools";
import { AssistantSidebar } from "../../../components/ai/AssistantSidebar";
import { usePdfOutline } from "../hooks/usePdfOutline";
import { usePdfSession } from "../hooks/usePdfSession";
import { PdfOutlineTree } from "./PdfOutlineTree";
import { PdfThumbnailList } from "./PdfThumbnailList";
import type { PdfBookmark } from "../types/pdfBookmark";
import { DocumentSearchPanel } from "../../search";
import { usePdfSearch } from "../search";
import { PdfBookmarkPanel } from "./PdfBookmarkPanel";
import { AnnotationNotesPanel } from "../../annotations";
import { usePdfWorkspace } from "../hooks/usePdfWorkspace";

type PdfSidebarProps = {
  currentPage: number;
  onNavigatePage: (page: number) => void;
  onNavigateOutline: (page: number, title: string) => void;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onResizeStart?: (event: PointerEvent) => void;
  bookmarks: PdfBookmark[];
  assistantText: string;
  assistantRequest: AssistantActionRequest | null;
};

export function PdfSidebar({ currentPage, onNavigatePage, onNavigateOutline, collapsed, onToggleCollapsed, onResizeStart, bookmarks, assistantText, assistantRequest }: PdfSidebarProps) {
  const { document } = usePdfSession();
  const search = usePdfSearch();
  const workspace = usePdfWorkspace();
  const [tab, setTab] = useState("pages");
  const outline = usePdfOutline(document, currentPage);
  useEffect(() => { if (assistantRequest) setTab("assistant"); }, [assistantRequest]);
  const bookmarkCounts = new Map<number, number>();
  bookmarks.forEach((bookmark) => bookmarkCounts.set(bookmark.page, (bookmarkCounts.get(bookmark.page) ?? 0) + 1));
  return <WorkspaceTools tools={[
    { id: "page-tools", label: "页面（待开发）", icon: <GalleryVerticalEnd size={20} />, disabled: true },
    { id: "search", label: "搜索", icon: <Search size={20} /> },
    { id: "pages", label: "缩略图", icon: <Files size={20} /> },
    { id: "outline", label: "目录", icon: <ListTree size={20} /> },
    { id: "notes", label: "笔记", icon: <NotebookPen size={20} /> },
    { id: "bookmarks", label: "书签", icon: <Bookmark size={20} /> },
    { id: "assistant", label: "AI 小助手", icon: <Bot size={20} /> },
  ]} active={tab} collapsed={collapsed} onSelect={setTab} onToggleCollapsed={onToggleCollapsed} onResizeStart={onResizeStart}>
    <div className="workspace-feature-body" hidden={tab === "assistant"}>
      {tab === "search" && <DocumentSearchPanel search={search} placeholder="搜索当前 PDF" />}
      {tab === "notes" && <AnnotationNotesPanel annotations={workspace.annotations} onUpdate={workspace.updateAnnotation} onDelete={workspace.deleteAnnotation} onMerge={workspace.mergeAnnotations} selectedAnnotationId={workspace.selectedAnnotationId} onSelect={workspace.navigateToAnnotation} />}
      {tab === "bookmarks" && <PdfBookmarkPanel />}
      {document && tab === "pages" && <PdfThumbnailList document={document} currentPage={currentPage} onNavigate={onNavigatePage} bookmarkCounts={bookmarkCounts} />}
      {document && tab === "outline" && <PdfOutlineTree items={outline.outline} loading={outline.loading} expanded={outline.expanded} activeId={outline.activeId} itemRefs={outline.itemRefs} onToggle={outline.toggleExpanded} onNavigate={onNavigateOutline} />}
    </div>
    <div className="workspace-assistant-host" hidden={tab !== "assistant"}>
      <AssistantSidebar embedded actionRequest={assistantRequest} contextText={assistantText} contextLabel="PDF 选中文本" emptyContextHint="在 PDF 中选中文字，右键选择“询问大模型”以带入上下文。" collapsed={false} onToggleCollapsed={onToggleCollapsed} />
    </div>
  </WorkspaceTools>;
}
