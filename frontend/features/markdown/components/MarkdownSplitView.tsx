import { MarkdownEditor } from "./MarkdownEditor";
import { MarkdownPreview } from "./MarkdownPreview";
import { useMarkdownWorkspace } from "../hooks/useMarkdownWorkspace";
import { useMarkdownScrollSync } from "../hooks/useMarkdownScrollSync";

export function MarkdownSplitView() {
  const workspace = useMarkdownWorkspace();
  const scroll = useMarkdownScrollSync({
    content: workspace.content,
    navigationRequest: workspace.outlineNavigationRequest,
    viewMode: workspace.viewMode,
  });

  return (
    <div className={`markdown-content markdown-view-${workspace.viewMode}`}>
      {workspace.viewMode !== "realtime" && <MarkdownEditor editorRef={scroll.editorRef} onScroll={scroll.onEditorScroll} />}
      {workspace.viewMode !== "source" && (
        <MarkdownPreview nodes={workspace.nodes} scrollRef={scroll.previewRef} onScroll={scroll.onPreviewScroll} />
      )}
    </div>
  );
}
